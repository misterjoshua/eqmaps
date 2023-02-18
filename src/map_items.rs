use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    fn parse(x: &str, y: &str, z: &str) -> Result<Self, anyhow::Error> {
        Ok(Point {
            x: x.parse()?,
            y: y.parse()?,
            z: z.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn parse(r: &str, g: &str, b: &str) -> Result<Self, anyhow::Error> {
        Ok(Color {
            r: r.parse()?,
            g: g.parse()?,
            b: b.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct PointItem {
    pub point: Point,
    pub color: Color,
    pub label: String,
}

#[derive(Debug)]
pub struct LineItem {
    pub from: Point,
    pub to: Point,
    pub color: Color,
}

#[derive(Debug)]
pub enum MapItem {
    PointItem(PointItem),
    LineItem(LineItem),
}

#[derive(Debug)]
pub struct MapItems {
    pub items: Vec<MapItem>,
}

impl MapItems {
    pub async fn load_from_files<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Result<Self, anyhow::Error> {
        let mut items = Vec::new();

        for path in paths {
            let map_items = MapItems::load_from_file(path).await?;
            for item in map_items.items {
                items.push(item);
            }
        }

        Ok(MapItems { items })
    }

    pub async fn load_from_file(path: &Path) -> Result<Self, anyhow::Error> {
        let file = File::open(path)?;

        let lines = BufReader::new(file).lines();
        let items = lines
            .filter_map(|line| {
                if let Ok(line) = line {
                    MapItem::parse(&line).ok()
                } else {
                    None
                }
            })
            .collect();

        Ok(MapItems { items })
    }
}

impl MapItem {
    fn parse(line: &str) -> Result<MapItem, anyhow::Error> {
        let first_char = line
            .chars()
            .nth(0)
            .ok_or_else(|| anyhow!("Missing line identifier"))?;

        let item = match first_char {
            'P' => MapItem::PointItem(PointItem::parse(&line)?),
            'L' => MapItem::LineItem(LineItem::parse(&line)?),
            _ => return Err(anyhow!("Unrecognized line identifier {}", first_char)),
        };

        Ok(item)
    }
}

impl PointItem {
    /// Parses a PointItem from a map file line.
    /// P 78.2306, -50.5124, 0.0020, 255, 0, 0, 3, to_The_Steamfont_Mountains
    fn parse(line: &str) -> Result<Self, anyhow::Error> {
        let (_, line) = line
            .split_once(' ')
            .ok_or_else(|| anyhow!("No line content"))?;

        let segments: Vec<&str> = LINE_CONTENT_SEPARATOR.split(line).collect();

        let [x, y, z, r, g, b, _point_type, label] = segments[..] else {
            return Err(anyhow!("Not enough line content segments"));
        };

        Ok(PointItem {
            point: Point::parse(x, y, z)?,
            color: Color::parse(r, g, b)?,
            label: String::from(label),
        })
    }
}

impl LineItem {
    /// Parses a LineItem from a map file line.
    /// L 1000.0, 0.0, 0.0, 1000.0, -50.0, 0.0, 255, 0, 0
    fn parse(line: &str) -> Result<Self, anyhow::Error> {
        let (_, line) = line
            .split_once(' ')
            .ok_or_else(|| anyhow!("No line content"))?;

        let segments: Vec<&str> = LINE_CONTENT_SEPARATOR.split(line).collect();

        let [fx, fy, fz, tx, ty, tz, r, g, b] = segments[..] else {
            return Err(anyhow!("Not enough line content segments"));
        };

        Ok(LineItem {
            from: Point::parse(fx, fy, fz)?,
            to: Point::parse(tx, ty, tz)?,
            color: Color::parse(r, g, b)?,
        })
    }
}

lazy_static! {
    static ref LINE_CONTENT_SEPARATOR: Regex = Regex::new(",\\s+").unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{map_items::MapItem};
    use std::assert_matches::assert_matches;

    #[test]
    fn parsing_point() {
        let map_item = MapItem::parse(
            "P 78.2306, -50.5124, 0.0020, 255, 254, 253, 3, to_The_Steamfont_Mountains",
        );

        assert_matches!(map_item, Ok(MapItem::PointItem(_)));

        if let Ok(MapItem::PointItem(point)) = map_item {
            assert_eq!(point.point.x, 78.2306);
            assert_eq!(point.point.y, -50.5124);
            assert_eq!(point.point.z, 0.0020);
            assert_eq!(point.color.r, 255);
            assert_eq!(point.color.g, 254);
            assert_eq!(point.color.b, 253);
            assert_eq!(point.label, "to_The_Steamfont_Mountains")
        }
    }

    #[test]
    fn parsing_line() {
        let map_item = MapItem::parse("L 1000.0, 1.1, 2.2, 1000.0, -50.0, 3.3, 255, 254, 253");

        assert_matches!(map_item, Ok(MapItem::LineItem(_)));

        if let Ok(MapItem::LineItem(line)) = map_item {
            assert_eq!(line.from.x, 1000.0);
            assert_eq!(line.from.y, 1.1);
            assert_eq!(line.from.z, 2.2);
            assert_eq!(line.to.x, 1000.0);
            assert_eq!(line.to.y, -50.0);
            assert_eq!(line.to.z, 3.3);
            assert_eq!(line.color.r, 255);
            assert_eq!(line.color.g, 254);
            assert_eq!(line.color.b, 253);
        }
    }
}
