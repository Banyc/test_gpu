use std::num::NonZeroUsize;

use math::{
    matrix::{ArrayMatrixBuf, Size},
    vector::Vector,
};
use strict_num::FiniteF64;

pub type Point = Vector<4>;
pub type PointMatrix = ArrayMatrixBuf<f64, 4>;
pub fn point_size() -> Size {
    Size {
        rows: NonZeroUsize::new(4).unwrap(),
        cols: NonZeroUsize::new(1).unwrap(),
    }
}
pub type TransformMatrix = ArrayMatrixBuf<f64, 16>;
pub fn transform_size() -> Size {
    Size {
        rows: NonZeroUsize::new(4).unwrap(),
        cols: NonZeroUsize::new(4).unwrap(),
    }
}

pub fn point(var: [f64; 3]) -> Point {
    let dims = [var[0], var[1], var[2], 1.].map(|x| FiniteF64::new(x).unwrap());
    Vector::new(dims)
}

pub fn zero() -> TransformMatrix {
    let data = [0.; 16];
    TransformMatrix::new(transform_size(), data)
}
pub fn scale(var: [f64; 3]) -> TransformMatrix {
    let data = [
        var[0], 0., 0., 0., //
        0., var[1], 0., 0., //
        0., 0., var[2], 0., //
        0., 0., 0., 1., //
    ];
    TransformMatrix::new(transform_size(), data)
}
pub fn translate(var: [f64; 3]) -> TransformMatrix {
    let data = [
        1., 0., 0., var[0], //
        0., 1., 0., var[1], //
        0., 0., 1., var[2], //
        0., 0., 0., 1., //
    ];
    TransformMatrix::new(transform_size(), data)
}
pub fn rotate(axises: [f64; 3], angle: f64) -> TransformMatrix {
    let cos = angle.cos();
    let cos_com = 1. - angle.cos();
    let sin = angle.sin();
    let a = axises;
    let data = [
        // row
        cos + (a[0].powi(2) * cos_com),
        (a[0] * a[1] * cos_com) - (a[2] * sin),
        (a[0] * a[2] * cos_com) + (a[1] * sin),
        0.,
        // row
        (a[0] * a[1] * cos_com) + (a[2] * sin),
        cos + (a[1].powi(2) * cos_com),
        (a[1] * a[2] * cos_com) + (a[0] * sin),
        0.,
        // row
        (a[0] * a[2] * cos_com) + (a[1] * sin),
        (a[1] * a[2] * cos_com) - (a[0] * sin),
        cos + (a[2].powi(2) * cos_com),
        0.,
        // row
        0.,
        0.,
        0.,
        1.,
    ];
    TransformMatrix::new(transform_size(), data)
}

#[cfg(test)]
mod tests {
    use math::matrix::Matrix;

    use super::*;

    #[test]
    fn test_transform() {
        let trans = translate([1., 2., 3.]);
        let scale = scale([2., 2., 2.]);
        let mut m = zero();
        trans.mul_matrix_in(&scale, &mut m);
        let expected = TransformMatrix::new(
            transform_size(),
            [
                2., 0., 0., 1., //
                0., 2., 0., 2., //
                0., 0., 2., 3., //
                0., 0., 0., 1., //
            ],
        );
        assert!(m.closes_to(&expected));

        let point = point([1., 2., 3.]);
        let mut p: PointMatrix = point.into();
        m.mul_matrix_in(&p.clone(), &mut p);
        dbg!(&p);
        assert!(p.closes_to(&ArrayMatrixBuf::new(point_size(), [3., 6., 9., 1.])));
        let _ = Point::try_from_matrix(p).unwrap();
    }
}
