#![doc = include_str!("../README.md")]

use nalgebra::{DMatrix, DVector, SimdComplexField};
use std::marker::PhantomData;

/// Execute TOPSIS calculation with given parameters.
///
/// `criteria_weights` is a vector that contains the weight of each criterion.\
/// `criteria_types` is a vector that contains the type of each criterion.\
/// `alternatives` is a flat matrix constructed in column major order that contains the data.\
/// `Vec<Alternative>` is a vector that contains the ranking of the alternatives along with its data in descending order.
pub fn calculate(
  criteria_weights: &[f64],
  criteria_types: &[bool],
  alternatives: &[f64],
) -> Vec<Alternative> {
  let ncols = criteria_weights.len();
  let data_length = alternatives.len();

  assert_eq!(
    ncols,
    criteria_types.len(),
    "The length of the criteria_weights and the criteria_types must be the same"
  );
  assert_eq!(
    0,
    data_length % ncols,
    "The number of columns in alternatives must be equal to the length of the criteria_types"
  );

  let nrows = data_length / ncols;
  let distance: RawDistance = alternatives
    .chunks_exact(nrows)
    .zip(criteria_weights)
    .map(|(col, weight)| {
      let mut col = DVector::<f64>::from_column_slice(col);
      let norm = col.norm();
      for v in col.iter_mut() {
        *v = v.simd_unscale(norm).simd_scale(*weight);
      }

      col
    })
    .zip(criteria_types)
    .map(|(col, is_benefit)| {
      let (max, min) = (col.max(), col.min());

      let (mut pisi, mut nisi) = if *is_benefit { (max, min) } else { (min, max) };
      pisi *= -1.0;
      nisi *= -1.0;

      let cap = col.len();
      let (mut pis, mut nis) = (Vec::with_capacity(cap), Vec::with_capacity(cap));

      for value in col.iter() {
        pis.push((value + pisi).powf(2.0));
        nis.push((value + nisi).powf(2.0));
      }

      (pis, nis)
    })
    .collect();

  let positive_distance = DMatrix::<f64>::from_column_slice(nrows, ncols, &distance.positive);
  let negative_distance = DMatrix::<f64>::from_column_slice(nrows, ncols, &distance.negative);
  drop(distance);

  let mut result: Vec<Alternative> = positive_distance
    .row_iter()
    .zip(negative_distance.row_iter())
    .map(|(p_row, n_row)| (p_row.sum().simd_sqrt(), n_row.sum().simd_sqrt()))
    .enumerate()
    .map(|(id, (pdv, ndv))| Alternative::new(ndv / (pdv + ndv), id))
    .collect();

  result.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
  result
}

#[derive(Default, Debug)]
/// A structure to represent the result data
///
/// `value` is the result value.\
/// `id` is the id of the data.
pub struct Alternative {
  pub value: f64,
  pub id: usize,
  _marker: PhantomData<f64>,
}

impl Alternative {
  fn new(value: f64, id: usize) -> Alternative {
    Alternative {
      value,
      id,
      _marker: PhantomData,
    }
  }
}

#[derive(Default)]
struct RawDistance {
  positive: Vec<f64>,
  negative: Vec<f64>,
}

impl FromIterator<(Vec<f64>, Vec<f64>)> for RawDistance {
  fn from_iter<T: IntoIterator<Item = (Vec<f64>, Vec<f64>)>>(iter: T) -> Self {
    // TODO: turn it to row major order
    let mut distance = RawDistance::default();

    for mut vec in iter {
      distance.positive.append(&mut vec.0);
      distance.negative.append(&mut vec.1);
    }

    distance
  }
}

mod test {
  #[cfg(test)]
  mod tests {
    use crate::calculate;

    #[test]
    fn it_ranks_correctly() {
      let result = calculate(
        &[0.64339, 0.28284, 0.07377],
        &[true, true, true],
        &[
          80.0, 70.0, 91.0, 90.0, 80.0, 71.0, 90.0, 78.0, 0.0, 1.0, 0.0, 4.0,
        ],
      );

      let rs: Vec<usize> = result.into_iter().map(|e| e.id).collect();

      assert_eq!(rs, &[3, 2, 0, 1]);
    }
  }
}
