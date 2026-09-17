use euclid::default::Rect;
use wgame::glam::Vec2;

pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect<f32> {
    Rect::new((x, y).into(), (width.max(0.0), height.max(0.0)).into())
}

pub fn contains(rect: Rect<f32>, point: Vec2) -> bool {
    rect.contains(point.to_array().into())
}

/// Row-major cells, choosing the grid closest to the requested cell aspect.
pub fn grid(size: Vec2, count: usize, aspect: f32) -> Vec<Rect<f32>> {
    if count == 0
        || !size.is_finite()
        || size.min_element() <= 0.0
        || !aspect.is_finite()
        || aspect <= 0.0
    {
        return Vec::new();
    }
    let columns = (1..=count)
        .min_by(|&a, &b| {
            let score = |cols: usize| {
                let rows = count.div_ceil(cols);
                let cell_aspect = (size.x / cols as f32) / (size.y / rows as f32);
                (cell_aspect / aspect).ln().abs()
            };
            score(a).total_cmp(&score(b))
        })
        .unwrap();
    let cell = size / Vec2::new(columns as f32, count.div_ceil(columns) as f32);
    (0..count)
        .map(|i| {
            rect(
                (i % columns) as f32 * cell.x,
                (i / columns) as f32 * cell.y,
                cell.x,
                cell.y,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grids_fit_wide_tall_and_small_windows() {
        for size in [
            Vec2::new(1280.0, 680.0),
            Vec2::new(320.0, 640.0),
            Vec2::splat(1.0),
        ] {
            for count in [0, 4, 10, 24, 26, 33] {
                let cells = grid(size, count, 2.0);
                assert_eq!(cells.len(), count);
                for (i, cell) in cells.iter().enumerate() {
                    assert!(cell.min_x() >= 0.0 && cell.min_y() >= 0.0);
                    assert!(cell.max_x() <= size.x + 0.001 && cell.max_y() <= size.y + 0.001);
                    assert!(
                        cells[..i]
                            .iter()
                            .all(|other| !cell.inflate(-0.0001, -0.0001).intersects(other))
                    );
                }
            }
        }
        assert!(grid(Vec2::ZERO, 4, 1.0).is_empty());
    }
}
