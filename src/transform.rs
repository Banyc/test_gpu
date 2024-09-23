use std::num::NonZeroUsize;

use math::{
    matrix::{ArrayMatrix, Size},
    vector::{Vector, VectorMut},
};

pub type Point = [f64; 4];
pub type PointMatrix = ArrayMatrix<f64, 4>;
pub fn point_size() -> Size {
    Size {
        rows: NonZeroUsize::new(4).unwrap(),
        cols: NonZeroUsize::new(1).unwrap(),
    }
}
pub type TransformMatrix = ArrayMatrix<f64, 16>;
pub fn transform_size() -> Size {
    Size {
        rows: NonZeroUsize::new(4).unwrap(),
        cols: NonZeroUsize::new(4).unwrap(),
    }
}

pub fn point(var: [f64; 3]) -> Point {
    [var[0], var[1], var[2], 1.]
}

pub fn identity() -> TransformMatrix {
    TransformMatrix::identity(transform_size().rows)
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
pub fn rotate(mut axises: [f64; 3], angle: f64) -> TransformMatrix {
    let cos = angle.cos();
    let cos_com = 1. - angle.cos();
    let sin = angle.sin();
    axises.normalize();
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
        (a[1] * a[2] * cos_com) - (a[0] * sin),
        0.,
        // row
        (a[0] * a[2] * cos_com) - (a[1] * sin),
        (a[1] * a[2] * cos_com) + (a[0] * sin),
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
pub fn change_of_space(i: [f64; 3], j: [f64; 3], k: [f64; 3], h: [f64; 3]) -> TransformMatrix {
    let data = [
        i[0], i[1], i[2], 0., //
        j[0], j[1], j[2], 0., //
        k[0], k[1], k[2], 0., //
        0., 0., 0., 1., //
    ];
    let change_of_axises = ArrayMatrix::new(transform_size(), data);
    let origin_translate = translate(h.map(|x| -x));
    change_of_axises.mul_matrix_square(&origin_translate)
}
pub fn look_at(h: [f64; 3], target: [f64; 3], up: [f64; 3]) -> TransformMatrix {
    let mut k: [f64; 3] = h;
    k.sub(&target);
    k.normalize();
    let mut i = up.cross(&k);
    i.normalize();
    let j = k.cross(&i);
    change_of_space(i, j, k, h)
}
pub fn orthographic(
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    near: f64,
    far: f64,
) -> TransformMatrix {
    let data = [
        // row
        2. / (right - left),
        0.,
        0.,
        0.,
        // row
        0.,
        2. / (top - bottom),
        0.,
        0.,
        // row
        0.,
        0.,
        -2. / (far - near),
        0.,
        // row
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        -(far + near) / (far - near),
        1.,
    ];
    TransformMatrix::new(transform_size(), data)
}
pub fn perspective(fov: f64, aspect: f64, near: f64, far: f64) -> TransformMatrix {
    let tan_inv = 1. / f64::tan(fov / 2.);
    let data = [
        // row
        (1. / aspect) * tan_inv,
        0.,
        0.,
        0.,
        // row
        0.,
        tan_inv,
        0.,
        0.,
        // row
        0.,
        0.,
        -(far + near) / (far - near),
        -(2. * far * near) / (far - near),
        // row
        0.,
        0.,
        -1.,
        0.,
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
        let m = trans.mul_matrix_square(&scale);
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
        let mut p: PointMatrix = point.into_matrix().unwrap();
        m.mul_matrix_in(&p.clone(), &mut p);
        dbg!(&p);
        assert!(p.closes_to(&ArrayMatrix::new(point_size(), [3., 6., 9., 1.])));
        Point::try_from(p).unwrap();
    }
}
