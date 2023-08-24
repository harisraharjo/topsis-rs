use nalgebra::{DMatrix, SimdComplexField};

///
pub fn topsis(criteria_weights: &[f64], is_benefits: &[bool], raw_matrix: &[f64]) -> Rank {
  let ncols = criteria_weights.len();
  let nrows = raw_matrix.len() / ncols;

  let mut matrix = DMatrix::<f64>::from_column_slice(nrows, ncols, raw_matrix);

  let distance: RawDistance = matrix
    .column_iter_mut()
    .zip(criteria_weights)
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

  let rc: Rank = positive_distance
    .row_iter()
    .zip(negative_distance.row_iter())
    .map(|(p_row, n_row)| (p_row.sum().simd_sqrt(), n_row.sum().simd_sqrt()))
    // .enumerate()
    .map(|(pdv, ndv)| ndv / (pdv + ndv))
    .collect();

  rc
}

#[derive(Debug, Default)]
pub struct Alternative {
  value: f64,
  id: usize,
}

impl Alternative {
  fn new(value: f64, id: usize) -> Alternative {
    Alternative { value, id }
  }
}

#[derive(Debug, Default)]
pub struct Rank {
  alternatives: Vec<Alternative>,
}

impl IntoIterator for Rank {
  type Item = Alternative;

  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.alternatives.into_iter()
  }
}

impl FromIterator<f64> for Rank {
  fn from_iter<T: IntoIterator<Item = f64>>(rcs_values: T) -> Self {
    let mut rank = Rank {
      alternatives: rcs_values
        .into_iter()
        .enumerate()
        .map(|(id, value)| Alternative::new(value, id))
        .collect(),
    };

    rank
      .alternatives
      .sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());

    rank
  }
}

#[derive(Clone, PartialEq, Debug, Default)]
struct RawDistance {
  positive: Vec<f64>,
  negative: Vec<f64>,
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
    distance.negative.shrink_to_fit();
    distance
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_rank_alternatives_correctly() {
    let result: Vec<usize> = topsis(
      &[0.64339f64, 0.28284f64, 0.07377f64],
      &[true, true, true],
      &[
        80f64, 70f64, 91f64, 90f64, 80f64, 71f64, 90f64, 78f64, 0f64, 1f64, 0f64, 4f64,
      ],
    )
    .into_iter()
    .map(|e| e.id)
    .collect();

    assert_eq!(result, &[3usize, 2usize, 0usize, 1usize]);
  }
}
