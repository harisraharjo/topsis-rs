use nalgebra::{DMatrix, SimdComplexField};

#[derive(Clone, PartialEq, Debug, Default)]
struct RawDistance {
  pub(crate) positive: Vec<f64>,
  pub(crate) negative: Vec<f64>,
}

impl FromIterator<Vec<(f64, f64)>> for RawDistance {
  fn from_iter<T: IntoIterator<Item = Vec<(f64, f64)>>>(iter: T) -> Self {
    let mut distance = RawDistance::default();

    for vec in iter {
      for value in vec {
        distance.positive.push(value.0);
        distance.negative.push(value.1);
      }
    }

    distance.positive.shrink_to_fit();
    distance
  }
}

pub fn topsis(criteria_weights: &[f64], is_benefits: &[bool], raw_matrix: &[f64]) -> Vec<f64> {
  let ncols = criteria_weights.len();
  let nrows = raw_matrix.len() / ncols;

  let mut matrix = DMatrix::<f64>::from_column_slice(nrows, ncols, raw_matrix);

  let distance: RawDistance = matrix
    .column_iter_mut()
    .zip(criteria_weights)
    // .zip(is_benefits)
    .map(|(mut col, weight)| {
      let norm = col.norm();
      for v in col.iter_mut() {
        *v = v.simd_unscale(norm).simd_scale(*weight);
      }

      col
    })
    .zip(is_benefits)
    .map(|(col, is_benefit)| {
      let (max, min) = (col.max(), col.min());

      //TODO: try to remove branches
      let (mut pis, mut nis) = if *is_benefit { (max, min) } else { (min, max) };
      pis *= -1.0;
      nis *= -1.0;

      // TODO: create a separate type for `Vec<(f64, f64)>` and implement Extend to it;
      let result: Vec<(f64, f64)> = col
        .iter()
        .map(|e| ((e + pis).powf(2.0), (e + nis).powf(2.0)))
        .collect();

      result
    })
    .collect();

  drop(matrix);

  let positive_distance = DMatrix::<f64>::from_column_slice(nrows, ncols, &distance.positive);
  let negative_distance = DMatrix::<f64>::from_column_slice(nrows, ncols, &distance.negative);
  drop(distance);

  let mut rc: Vec<f64> = positive_distance
    .row_iter()
    .zip(negative_distance.row_iter())
    .map(|(p_row, n_row)| (p_row.sum().simd_sqrt(), n_row.sum().simd_sqrt()))
    .map(|(pdv, ndv)| ndv / (pdv + ndv))
    .collect();

  rc.sort_by(|a, b| b.partial_cmp(a).unwrap());

  rc
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_rank_alternatives_correctly() {
    let result = topsis(
      &[0.64339f64, 0.28284f64, 0.07377f64],
      &[true, true, true],
      &[
        80f64, 70f64, 91f64, 90f64, 80f64, 71f64, 90f64, 78f64, 0f64, 1f64, 0f64, 4f64,
      ],
    );

    assert_eq!(
      result,
      &[
        0.8311594494103931,
        0.5511369430527522,
        0.32943698268828747,
        0.148034942264447,
      ]
    );
  }
}
