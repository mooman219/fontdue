use crate::raw::RawPoint;
use alloc::vec::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// Absolute X coordinate.
    pub x: f32,
    /// Absolute Y coordinate.
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x,
            y,
        }
    }

    pub fn new_raw(p: &RawPoint) -> Point {
        Point {
            x: p.x,
            y: p.y,
        }
    }

    pub fn lerp(t: f32, a: &Point, b: &Point) -> Point {
        Point {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        }
    }

    pub fn lerp_raw(t: f32, a: &RawPoint, b: &RawPoint) -> Point {
        Point {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        }
    }

    pub fn midpoint_raw(a: &RawPoint, b: &RawPoint) -> Point {
        Point {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }

    /// Scales the X and Y components by the given scale.
    pub fn scale(&self, scale: f32) -> Point {
        Point {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    /// Mirrors the point ofer the horizontal line at the given y.
    pub fn mirror_x(&self, y: f32) -> Point {
        Point {
            x: self.x,
            y: self.y + ((y - self.y) * 2.0),
        }
    }

    /// Offsets the point by the given x and y.
    pub fn offset(&self, x: f32, y: f32) -> Point {
        Point {
            x: self.x + x,
            y: self.y + y,
        }
    }
}

const INVALID_COORD: u32 = !0u32;

pub struct Geometry {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}

impl Geometry {
    pub fn line(a: Point, b: Point) -> Geometry {
        Geometry {
            a,
            b,
            c: Point::new(f32::from_bits(INVALID_COORD), 0.0),
        }
    }

    pub fn curve(a: Point, b: Point, c: Point) -> Geometry {
        Geometry {
            a,
            b,
            c,
        }
    }

    pub fn is_line(&self) -> bool {
        self.c.x.to_bits() == INVALID_COORD
    }

    /// Scales the X and Y components by the given scale.
    pub fn scale(&self, scale: f32) -> Geometry {
        Geometry {
            a: self.a.scale(scale),
            b: self.b.scale(scale),
            c: if self.c.x.to_bits() == INVALID_COORD {
                self.c
            } else {
                self.c.scale(scale)
            },
        }
    }

    /// Mirrors the geometry ofer the horizontal line at the given y.
    pub fn mirror_x(&self, y: f32) -> Geometry {
        Geometry {
            a: self.a.mirror_x(y),
            b: self.b.mirror_x(y),
            c: if self.c.x.to_bits() == INVALID_COORD {
                self.c
            } else {
                self.c.mirror_x(y)
            },
        }
    }

    /// Offsets the geometry by the given x and y.
    pub fn offset(&self, x: f32, y: f32) -> Geometry {
        Geometry {
            a: self.a.offset(x, y),
            b: self.b.offset(x, y),
            c: if self.c.x.to_bits() == INVALID_COORD {
                self.c
            } else {
                self.c.offset(x, y)
            },
        }
    }
}

/// Converts a series of raw points over any number of contours into discrete geometry.
pub fn to_geometry(points: &[RawPoint]) -> Vec<Geometry> {
    // TODO: This can be done with fewer allocations.
    let mut geometry = Vec::new();
    let mut contour = Vec::new();
    for point in points {
        contour.push(*point);
        if point.end_point {
            for (index, current) in (&contour).iter().enumerate() {
                let next = next(index, &contour);
                let previous = previous(index, &contour);
                if !current.on_curve() {
                    // Curve. We're off the curve, find the on-curve positions for the previous
                    // and next points then make a curve out of that.
                    let previous = if previous.on_curve() {
                        Point::new_raw(&previous)
                    } else {
                        Point::midpoint_raw(&previous, current)
                    };
                    let next = if next.on_curve() {
                        Point::new_raw(&next)
                    } else {
                        Point::midpoint_raw(current, &next)
                    };
                    let current = Point::new_raw(current);
                    geometry.push(Geometry::curve(previous, current, next));
                } else if next.on_curve() {
                    // Line. Both the current and the next point are on the curve, it's a line.
                    let current = Point::new_raw(current);
                    let next = Point::new_raw(&next);
                    geometry.push(Geometry::line(current, next));
                } else {
                    // Do nothing. The current point is on the curve but the next one isn't, so the
                    // next point will end up drawing the curve that the current point is on.
                }
            }
            contour.clear();
        }
    }
    geometry
}

#[inline]
fn previous(index: usize, points: &[RawPoint]) -> RawPoint {
    if index == 0 {
        points[points.len() - 1]
    } else {
        points[index - 1]
    }
}

#[inline]
fn next(index: usize, points: &[RawPoint]) -> RawPoint {
    if index == points.len() - 1 {
        points[0]
    } else {
        points[index + 1]
    }
}
