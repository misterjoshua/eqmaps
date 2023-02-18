use anyhow::anyhow;
use std::{cmp::Ordering, path::Path};

use crate::map_items::{Color, LineItem, MapItem, MapItems, PointItem};

trait SvgDraw {
    fn svg(&self) -> String;
}

impl SvgDraw for LineItem {
    fn svg(&self) -> String {
        String::from(format!(
            "<path d=\"M {} {} L {} {}\" stroke=\"{}\" class=\"line-item\" />\n",
            self.from.x,
            self.from.y,
            self.to.x,
            self.to.y,
            self.color.svg()
        ))
    }
}

impl SvgDraw for PointItem {
    fn svg(&self) -> String {
        String::from(format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"{}\" class=\"point-item-circle\" />\n",
            self.point.x,
            self.point.y,
            self.color.svg()
        ))
    }
}

impl SvgDraw for Color {
    fn svg(&self) -> String {
        String::from(format!("rgb({},{},{})", self.r, self.g, self.b))
    }
}

impl SvgDraw for MapItems {
    fn svg(&self) -> String {
        let view_box = map_view_box(&self);

        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\" xmlns=\"http://www.w3.org/2000/svg\">\n",
            view_box.2, view_box.3, view_box.0, view_box.1, view_box.2, view_box.3
        ));

        for item in self.items.iter() {
            let path = match item {
                MapItem::LineItem(line) => line.svg(),
                MapItem::PointItem(point) => point.svg(),
            };

            svg.push_str(path.as_str());
        }

        svg.push_str(&format!("</svg>\n"));

        svg
    }
}

pub fn map_draw(map_items: &MapItems, out_file: &Path) -> Result<(), anyhow::Error> {
    let svg = map_items.svg();

    let mut options = usvg::Options::default();
    options.fontdb.load_system_fonts();
    let rtree = usvg::Tree::from_data(&svg.as_bytes(), &options.to_ref())?;

    let fit_to = usvg::FitTo::Zoom(1.0);
    let pixmap_size = fit_to
        .fit_to(rtree.svg_node().size.to_screen_size())
        .unwrap();

    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or_else(|| anyhow!("Could not create a pixmap"))?;

    resvg::render(
        &rtree,
        fit_to,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .ok_or_else(|| anyhow!("Failed to render the SVG"))?;

    pixmap.save_png(out_file)?;

    Ok(())
}

pub fn map_view_box(map_items: &MapItems) -> (f32, f32, f32, f32) {
    let mut xs: Vec<f32> = vec![];
    let mut ys: Vec<f32> = vec![];

    map_items.items.iter().for_each(|item| match item {
        MapItem::LineItem(line) => {
            xs.push(line.from.x);
            ys.push(line.from.y);
            xs.push(line.to.x);
            ys.push(line.to.y);
        }
        MapItem::PointItem(point) => {
            xs.push(point.point.x);
            ys.push(point.point.y);
        }
    });

    xs.sort_by(float_ord);
    ys.sort_by(float_ord);

    let min_x = xs.first().unwrap_or(&0.0);
    let min_y = ys.first().unwrap_or(&0.0);
    let max_x = xs.last().unwrap_or(&0.0);
    let max_y = ys.last().unwrap_or(&0.0);

    return (*min_x, *min_y, max_x - min_x, max_y - min_y);
}

fn float_ord(a: &f32, b: &f32) -> Ordering {
    if a < b {
        Ordering::Less
    } else if b < a {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}
