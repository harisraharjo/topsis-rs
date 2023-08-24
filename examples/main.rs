use topsis::calculate;

fn main() {
  let ranking = calculate(
    &[0.64339, 0.28284, 0.07377],
    &[true, true, true],
    &[
      80.0, 70.0, 91.0, 90.0, 80.0, 71.0, 90.0, 78.0, 0.0, 1.0, 0.0, 4.0,
    ],
  );

  println!("{:#?}", ranking);

  let rs: Vec<usize> = ranking.into_iter().map(|e| e.id).collect();

  assert_eq!(rs, &[3, 2, 0, 1]);
}
