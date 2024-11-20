use std::iter::Peekable;

use macroquad::math::Rect;

struct SplitIter {
    index: usize,
    size: f32,
}

impl SplitIter {
    fn new(size: f32) -> Self {
        Self { index: 0, size }
    }
}

impl Iterator for SplitIter {
    type Item = (usize, f32);
    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        let n = self.index;
        Some((n, self.size / n as f32))
    }
}

struct GridSizeIter {
    x_iter: Peekable<SplitIter>,
    y_iter: Peekable<SplitIter>,
    aspect: f32,
}

impl GridSizeIter {
    fn new(w: f32, h: f32, aspect: f32) -> Self {
        Self {
            x_iter: SplitIter::new(w).peekable(),
            y_iter: SplitIter::new(h).peekable(),
            aspect,
        }
    }
}

impl Iterator for GridSizeIter {
    type Item = ((usize, usize), (f32, f32));
    fn next(&mut self) -> Option<Self::Item> {
        let (nx, sx) = *self.x_iter.peek().unwrap();
        let (ny, sy) = *self.y_iter.peek().unwrap();
        if sx >= sy * self.aspect {
            self.x_iter.next().unwrap();
        } else {
            self.y_iter.next().unwrap();
        }
        Some(((nx, ny), (sx, sy)))
    }
}

pub fn grid(size: (f32, f32), n: usize, aspect: f32) -> Vec<Vec<Rect>> {
    let (w, h) = size;
    let mut iter = GridSizeIter::new(w, h, aspect);
    let ((nx, ny), (sx, sy)) = loop {
        let item = iter.next().unwrap();
        let (nx, ny) = item.0;
        if nx * ny >= n {
            break item;
        }
    };
    let mut count = 0;
    let mut boxes = Vec::new();
    'outer: for iy in 0..ny {
        boxes.push(Vec::new());
        let line = boxes.last_mut().unwrap();
        for ix in 0..nx {
            line.push(Rect::new(sx * ix as f32, sy * iy as f32, sx, sy));
            count += 1;
            if count >= n {
                break 'outer;
            }
        }
    }
    boxes
}
