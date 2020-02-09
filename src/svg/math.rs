use super::point::Point;
use core::f64::consts::PI;

pub trait CurvePoint {
    fn at(&self, time: f64) -> Point;
}

pub struct SquareCurve {
    start: Point,
    p1: Point,
    end: Point,
}

impl SquareCurve {
    pub fn new(start: Point, p1: Point, end: Point) -> Self {
        SquareCurve { start, p1, end }
    }
}

impl CurvePoint for SquareCurve {
    fn at(&self, time: f64) -> Point {
        let diff = 1. - time;
        let square_t = time * time;
        let square_diff = diff * diff;
        self.start * square_diff + self.p1 * 2. * time * diff + self.end * square_t
    }
}

pub struct CubicCurve {
    start: Point,
    p1: Point,
    p2: Point,
    end: Point,
}

impl CubicCurve {
    pub fn new(start: Point, p1: Point, p2: Point, end: Point) -> Self {
        CubicCurve { start, p1, p2, end }
    }
}

impl CurvePoint for CubicCurve {
    fn at(&self, time: f64) -> Point {
        let diff = 1. - time;
        let square_t = time * time;
        let cube_t = square_t * time;
        let square_diff = diff * diff;
        let cube_diff = square_diff * diff;

        self.start * cube_diff
            + self.p1 * 3. * time * square_diff
            + self.p2 * 3. * square_t * diff
            + self.end * cube_t
    }
}

pub struct EllipseCurve {
    start_angle: f64,
    sweep_angle: f64,
    rx_abs: f64,
    ry_abs: f64,
    x_rad_rotation: f64,
    center_x: f64,
    center_y: f64,
}

impl EllipseCurve {
    pub fn new(
        start_angle: f64,
        sweep_angle: f64,
        rx_abs: f64,
        ry_abs: f64,
        x_rad_rotation: f64,
        center_x: f64,
        center_y: f64,
    ) -> Self {
        EllipseCurve {
            start_angle,
            sweep_angle,
            rx_abs,
            ry_abs,
            x_rad_rotation,
            center_x,
            center_y,
        }
    }
}

impl CurvePoint for EllipseCurve {
    fn at(&self, time: f64) -> Point {
        let angle = self.start_angle + self.sweep_angle * time;
        let ellipse_component_x = self.rx_abs * angle.cos();
        let ellipse_component_y = self.ry_abs * angle.sin();

        let point_x = self.x_rad_rotation.cos() * ellipse_component_x
            - self.x_rad_rotation.sin() * ellipse_component_y
            + self.center_x;
        let point_y = self.x_rad_rotation.sin() * ellipse_component_x
            + self.x_rad_rotation.cos() * ellipse_component_y
            + self.center_y;

        Point::new(point_x, point_y)
    }
}

pub fn ellipse_support_calc(
    current: Point,
    rx: f64,
    ry: f64,
    x_axis_rotation: f64,
    large_arc: bool,
    sweep: bool,
    end_x: f64,
    end_y: f64,
) -> (f64, f64, f64, f64, f64, f64, f64) {
    //calculations from: https://github.com/MadLittleMods/svg-curve-lib/

    let start_x = current.x;
    let start_y = current.y;

    let mut rx_abs = rx.abs();
    let mut ry_abs = ry.abs();
    let x_axis_rotation_mod_360 = x_axis_rotation % 360.0;
    let x_rad_rotation: f64 = x_axis_rotation_mod_360 * PI / 180.0;

    let dx = (start_x - end_x) / 2.;
    let dy = (start_y - end_y) / 2.;

    // Step #1: Compute transformedPoint
    let dx_rotated = x_rad_rotation.cos() * dx + x_rad_rotation.sin() * dy;
    let dy_rotated = -x_rad_rotation.sin() * dx + x_rad_rotation.cos() * dy;

    let radii_check = sqr(dx_rotated) / sqr(rx_abs) + sqr(dy_rotated) / sqr(ry_abs);
    if radii_check > 1.0 {
        rx_abs = radii_check.sqrt() * rx_abs;
        ry_abs = radii_check.sqrt() * ry_abs;
    }

    // Step #2: Compute transformedCenter
    let center_square_numerator =
        sqr(rx_abs) * sqr(ry_abs) - sqr(rx_abs) * sqr(dy_rotated) - sqr(ry_abs) * sqr(dx_rotated);
    let center_square_root_denom = sqr(rx_abs) * sqr(dy_rotated) + sqr(ry_abs) * sqr(dx_rotated);
    let mut center_radicand = center_square_numerator / center_square_root_denom;
    if center_radicand < 0. {
        center_radicand = 0.
    };

    let center_coef = {
        let sqrt = center_radicand.sqrt();
        if large_arc != sweep {
            sqrt
        } else {
            -sqrt
        }
    };
    let center_x_rotated = center_coef * (rx_abs * dy_rotated / ry_abs);
    let center_y_rotated = center_coef * (-ry_abs * dx_rotated / rx_abs);

    // Step #3: Compute center
    let center_x = x_rad_rotation.cos() * center_x_rotated
        - x_rad_rotation.sin() * center_y_rotated
        + ((start_x + end_x) / 2.);
    let center_y = x_rad_rotation.sin() * center_x_rotated
        + x_rad_rotation.cos() * center_y_rotated
        + ((start_y + end_y) / 2.);

    // Step #4: Compute start/sweep angles
    let start_vector_x = (dx_rotated - center_x_rotated) / rx_abs;
    let start_vector_y = (dy_rotated - center_y_rotated) / ry_abs;
    let start_vector = Point::new(start_vector_x, start_vector_y);
    let start_angle = angle_between(Point::new(1., 0.), start_vector);

    let end_vector_x = (-dx_rotated - center_x_rotated) / rx_abs;
    let end_vector_y = (-dy_rotated - center_y_rotated) / ry_abs;
    let end_vector = Point::new(end_vector_x, end_vector_y);
    let mut sweep_angle = angle_between(start_vector, end_vector);
    if !sweep && sweep_angle > 0. {
        sweep_angle -= 2. * PI;
    } else if sweep && sweep_angle < 0. {
        sweep_angle += 2. * PI;
    }
    sweep_angle = sweep_angle % (2. * PI);

    (
        start_angle,
        sweep_angle,
        rx_abs,
        ry_abs,
        x_rad_rotation,
        center_x,
        center_y,
    )
}

pub fn sqr(x: f64) -> f64 {
    x * x
}

pub fn angle_between(start: Point, end: Point) -> f64 {
    let p = start.x * end.x + start.y * end.y;
    let n = ((sqr(start.x) + sqr(start.y)) * (sqr(end.x) + sqr(end.y))).sqrt();
    let sign = if start.x * end.y - start.y * end.x < 0. {
        -1.
    } else {
        1.
    };
    let angle = sign * (p / n).acos();
    return angle;
}

const EPSILON: f64 = 0.05;
pub fn is_point_on_lane(lane_start: Point, lane_end: Point, p: &Point) -> bool {
    let vector = lane_end - lane_start;

    let left_part = if vector.x == 0. {
        0.
    } else {
        (p.x - lane_start.x) / vector.x
    };
    let right_part = if vector.y == 0. {
        0.
    } else {
        (p.y - lane_start.y) / vector.y
    };

    let is_on_lane = left_part - right_part;
    is_on_lane.abs() < EPSILON
}
