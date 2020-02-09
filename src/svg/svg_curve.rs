use svgtypes::{PathCommand, PathSegment};

use super::math::*;
use super::point::*;
use super::tick_timer::TickTimer;

pub enum LineTo {
    Fly(Point),
    Draw(Point),
    Erase(Point),
}

impl LineTo {
    fn new(point: Point, move_type: MoveType) -> Self {
        match move_type {
            MoveType::Fly => LineTo::Fly(point),
            MoveType::Draw => LineTo::Draw(point),
            MoveType::Erase => LineTo::Erase(point),
        }
    }
}

pub fn points_from_path_segments(
    path_segments: impl Iterator<Item = PathSegment>,
) -> impl Iterator<Item = LineTo> {
    let mut current_point = Point::ZERO;
    let mut prev_support_point_opt: Option<SupportPoint> = None;
    let mut path_start_point = Point::ZERO;
    let mut path_start_point_initialized = false;

    path_segments.flat_map(move |path_segment| {
        let point_iterator = calc_point_iterator(
            current_point,
            path_segment,
            prev_support_point_opt,
            path_start_point,
        );
        prev_support_point_opt = point_iterator.support_point();
        current_point = point_iterator.end_position();

        if !path_start_point_initialized && path_segment.cmd() != PathCommand::ClosePath {
            path_start_point_initialized = true;
            path_start_point = current_point;
        } else if path_segment.cmd() == PathCommand::ClosePath {
            path_start_point_initialized = false;
        }

        let move_type = point_iterator.move_type();
        point_iterator.map(move |point| LineTo::new(point, move_type))
    })
}

// === private members ===

#[derive(PartialEq, Copy, Clone)]
enum MoveType {
    Fly,
    Draw,
    Erase,
}

#[derive(Debug, Copy, Clone)]
struct SupportPoint {
    path_command: PathCommand,
    point: Point,
}

// === === === EMPTY === === ===
struct EmptyPointIterator {
    end: Point,
}

// === === === LINE === === ===
struct LinePointIterator {
    end: Point,
    move_type: MoveType,
    done: bool,
    support_point: Option<SupportPoint>,
}

impl LinePointIterator {
    fn new(end: Point, move_type: MoveType) -> Self {
        LinePointIterator {
            end,
            move_type,
            done: false,
            support_point: None,
        }
    }

    fn with_support(end: Point, move_type: MoveType, support_point: Option<SupportPoint>) -> Self {
        LinePointIterator {
            end,
            move_type,
            done: false,
            support_point,
        }
    }
}

// === === === CURVE === === ===
struct SquareCurvePointIterator {
    time: TickTimer,
    calc_formula: SquareCurve,
    support_point: Option<SupportPoint>,
}

struct CubicCurvePointIterator {
    time: TickTimer,
    calc_formula: CubicCurve,
    support_point: Option<SupportPoint>,
}

// === === === ELLIPSE === === ===
struct EllipsePointIterator {
    time: TickTimer,
    calc_formula: EllipseCurve,
    end: Point,
}

// === === === POINT ITERATOR === === ===
enum PointIterator {
    Empty(EmptyPointIterator),
    Line(LinePointIterator),
    SquareCurve(SquareCurvePointIterator),
    CubicCurve(CubicCurvePointIterator),
    EllipseCurve(EllipsePointIterator),
}

//todo: looks like I can remove one layer of abstraction!
impl PointIterator {
    //support point is always in absolute
    fn support_point(&self) -> Option<SupportPoint> {
        match self {
            PointIterator::Empty(_) => None,
            PointIterator::Line(iter) => iter.support_point,
            PointIterator::SquareCurve(iter) => iter.support_point,
            PointIterator::CubicCurve(iter) => iter.support_point,
            PointIterator::EllipseCurve(_) => None,
        }
    }

    fn end_position(&self) -> Point {
        match self {
            PointIterator::Empty(iter) => iter.end,
            PointIterator::Line(iter) => iter.end,
            PointIterator::SquareCurve(iter) => iter.calc_formula.at(1.0),
            PointIterator::CubicCurve(iter) => iter.calc_formula.at(1.0),
            PointIterator::EllipseCurve(iter) => iter.end,
        }
    }

    fn move_type(&self) -> MoveType {
        match self {
            PointIterator::Empty(_) => MoveType::Fly,
            PointIterator::Line(iter) => iter.move_type,
            PointIterator::SquareCurve(_) => MoveType::Draw,
            PointIterator::CubicCurve(_) => MoveType::Draw,
            PointIterator::EllipseCurve(_) => MoveType::Draw,
        }
    }
}

impl Iterator for PointIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PointIterator::Empty(_) => None,
            PointIterator::Line(iter) => {
                if iter.done {
                    None
                } else {
                    iter.done = true;
                    Some(iter.end)
                }
            }
            PointIterator::SquareCurve(iter) => {
                iter.time.next().map(|time| iter.calc_formula.at(time))
            }
            PointIterator::CubicCurve(iter) => {
                iter.time.next().map(|time| iter.calc_formula.at(time))
            }
            PointIterator::EllipseCurve(iter) => {
                iter.time.next().map(|time| iter.calc_formula.at(time))
            }
        }
    }
}

fn calc_point_iterator(
    current: Point,
    next_segment: PathSegment,
    prev_support_point_opt: Option<SupportPoint>,
    path_start_point: Point, //need that to implement ClosePath
) -> PointIterator {
    match next_segment {
        PathSegment::MoveTo { abs, x, y } => move_to(current, abs, x, y),
        PathSegment::LineTo { abs, x, y } => line_to(current, abs, x, y),
        PathSegment::HorizontalLineTo { abs, x } => {
            let miss_coord = if abs { current.y } else { 0. };
            line_to(current, abs, x, miss_coord)
        }
        PathSegment::VerticalLineTo { abs, y } => {
            let miss_coord = if abs { current.x } else { 0. };
            line_to(current, abs, miss_coord, y)
        }
        PathSegment::CurveTo {
            abs,
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => cubic_curve_to(current, abs, x1, y1, x2, y2, x, y, next_segment),
        PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => smooth_cubic_curve_to(
            current,
            abs,
            x2,
            y2,
            x,
            y,
            prev_support_point_opt,
            next_segment,
        ),
        PathSegment::Quadratic { abs, x1, y1, x, y } => {
            quadratic_curve_to(current, abs, x1, y1, x, y, next_segment)
        }
        PathSegment::SmoothQuadratic { abs, x, y } => {
            smooth_quadratic_curve_to(current, abs, x, y, prev_support_point_opt, next_segment)
        }
        PathSegment::EllipticalArc {
            abs,
            rx,
            ry,
            x_axis_rotation,
            large_arc,
            sweep,
            x,
            y,
        } => ellipse_curve_to(
            current,
            abs,
            rx,
            ry,
            x_axis_rotation,
            large_arc,
            sweep,
            x,
            y,
        ),
        PathSegment::ClosePath { abs: _ } => {
            line_to(current, true, path_start_point.x, path_start_point.y)
        }
    }
}

fn move_to(current: Point, abs: bool, x: f64, y: f64) -> PointIterator {
    let end_point = absolute_point_coord(current, abs, x, y);
    PointIterator::Line(LinePointIterator::new(end_point, MoveType::Fly))
}

fn line_to(current: Point, abs: bool, x: f64, y: f64) -> PointIterator {
    let end_point = absolute_point_coord(current, abs, x, y);
    PointIterator::Line(LinePointIterator::new(end_point, MoveType::Draw))
}

fn cubic_curve_to(
    current: Point,
    abs: bool,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x: f64,
    y: f64,
    next_segment: PathSegment,
) -> PointIterator {
    let time: TickTimer = Default::default();
    let p1 = absolute_point_coord(current, abs, x1, y1);
    let p2 = absolute_point_coord(current, abs, x2, y2);
    let end_point = absolute_point_coord(current, abs, x, y);
    let support_point = Some(SupportPoint {
        path_command: next_segment.cmd(),
        point: p2,
    });

    let p1_on_lane = is_point_on_lane(current, end_point, &p1);
    let p2_on_lane = is_point_on_lane(current, end_point, &p2);

    if p1_on_lane && p2_on_lane {
        PointIterator::Line(LinePointIterator::with_support(
            end_point,
            MoveType::Draw,
            support_point,
        ))
    } else {
        let calc_formula = CubicCurve::new(current, p1, p2, end_point);
        let cubic_curve_iterator = CubicCurvePointIterator {
            time,
            calc_formula,
            support_point,
        };
        PointIterator::CubicCurve(cubic_curve_iterator)
    }
}

fn smooth_cubic_curve_to(
    current: Point,
    abs: bool,
    x2: f64,
    y2: f64,
    x: f64,
    y: f64,
    prev_support_point_opt: Option<SupportPoint>,
    next_segment: PathSegment,
) -> PointIterator {
    let p1 = mirrored_point(current, abs, prev_support_point_opt, CurveType::Cubic);
    cubic_curve_to(current, abs, p1.x, p1.y, x2, y2, x, y, next_segment)
}

fn quadratic_curve_to(
    current: Point,
    abs: bool,
    x1: f64,
    y1: f64,
    x: f64,
    y: f64,
    next_segment: PathSegment,
) -> PointIterator {
    let time: TickTimer = Default::default();
    let p1 = absolute_point_coord(current, abs, x1, y1);
    let end_point = absolute_point_coord(current, abs, x, y);
    let support_point = Some(SupportPoint {
        path_command: next_segment.cmd(),
        point: Point { x: p1.x, y: p1.y },
    });

    let p1_on_lane = is_point_on_lane(current, end_point, &p1);
    if p1_on_lane {
        PointIterator::Line(LinePointIterator::with_support(
            end_point,
            MoveType::Draw,
            support_point,
        ))
    } else {
        let calc_formula = SquareCurve::new(current, p1, end_point);
        let square_curve_iterator = SquareCurvePointIterator {
            time,
            calc_formula,
            support_point,
        };
        PointIterator::SquareCurve(square_curve_iterator)
    }
}

fn smooth_quadratic_curve_to(
    current: Point,
    abs: bool,
    x: f64,
    y: f64,
    prev_support_point_opt: Option<SupportPoint>,
    next_segment: PathSegment,
) -> PointIterator {
    let p1 = mirrored_point(current, abs, prev_support_point_opt, CurveType::Quadratic);
    quadratic_curve_to(current, abs, p1.x, p1.y, x, y, next_segment)
}

fn ellipse_curve_to(
    current: Point,
    abs: bool,
    rx: f64,
    ry: f64,
    x_axis_rotation: f64,
    large_arc: bool,
    sweep: bool,
    end_x: f64,
    end_y: f64,
) -> PointIterator {
    let time: TickTimer = Default::default();

    let end_point = absolute_point_coord(current, abs, end_x, end_y);

    // If the endpoints are identical, then this is equivalent to omitting the elliptical arc segment entirely.
    if current == end_point {
        return PointIterator::Empty(EmptyPointIterator { end: end_point });
    }

    // If rx = 0 or ry = 0 then this arc is treated as a straight line segment joining the endpoints.
    if rx == 0. || ry == 0. {
        return line_to(current, abs, end_x, end_y);
    }

    let (start_angle, sweep_angle, rx_abs, ry_abs, x_rad_rotation, center_x, center_y) =
        ellipse_support_calc(
            current,
            rx,
            ry,
            x_axis_rotation,
            large_arc,
            sweep,
            end_point.x,
            end_point.y,
        );

    let calc_formula = EllipseCurve::new(
        start_angle,
        sweep_angle,
        rx_abs,
        ry_abs,
        x_rad_rotation,
        center_x,
        center_y,
    );
    PointIterator::EllipseCurve(EllipsePointIterator {
        time,
        calc_formula,
        end: end_point,
    })
}

fn absolute_point_coord(start: Point, abs: bool, x: f64, y: f64) -> Point {
    match abs {
        true => Point { x, y },
        false => Point { x, y } + start,
    }
}

enum CurveType {
    Cubic,
    Quadratic,
}

fn path_command_condition(prev_support_point: &SupportPoint, curve_type: CurveType) -> bool {
    match curve_type {
        CurveType::Cubic => {
            prev_support_point.path_command == PathCommand::SmoothCurveTo
                || prev_support_point.path_command == PathCommand::CurveTo
        }

        CurveType::Quadratic => {
            prev_support_point.path_command == PathCommand::SmoothQuadratic
                || prev_support_point.path_command == PathCommand::Quadratic
        }
    }
}

fn mirrored_point(
    current: Point,
    abs: bool,
    prev_support_point_opt: Option<SupportPoint>,
    curve_type: CurveType,
) -> Point {
    let mut mirrored_point = match prev_support_point_opt {
        Some(ref prev_support_point) if path_command_condition(prev_support_point, curve_type) => {
            current - prev_support_point.point
        }
        _ => Point::ZERO,
    };

    if abs {
        mirrored_point = mirrored_point + current;
    }

    mirrored_point
}
