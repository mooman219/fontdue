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
        Point::lerp_raw(0.5, a, b)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Geometry {
    Line(Point, Point),
    Curve(Point, Point, Point),
    None,
}

pub fn to_geometry(points: &[RawPoint]) -> Vec<Geometry> {
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
                    geometry.push(Geometry::Curve(previous, current, next));
                } else if next.on_curve() {
                    let current = Point::new_raw(current);
                    let next = Point::new_raw(&next);
                    // Line. Both the current and the next point are on the curve, it's a line.
                    geometry.push(Geometry::Line(current, next));
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
