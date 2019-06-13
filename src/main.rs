extern crate seq;
use seq::*;

extern crate gnuplot;
use gnuplot::*;

fn show(name: &str, x: &Vec<f64>, y: &Vec<f64>, (s_x, s_y): (f64, f64)) {
    let mut fg = Figure::new();
    fg.axes2d()
        .set_x_ticks(Some((Fix(s_x), 0)), &[], &[])
        .set_y_ticks(Some((Fix(s_y), 0)), &[], &[])
        .set_y_range(Fix(0.0), Fix(1.0))
        .set_x_range(Fix(0.0), Fix(1.0))
        .set_x_grid(true)
        .set_y_grid(true)
        .set_title(name, &[])
        .points(
            x,
            y,
            &[PointSymbol('O'), Color("#00000000"), PointSize(1.0)],
        );
    fg.show();
}

fn main() {
    let n = 16 * 16;
    let n_sqrt = 16;
    let do_show = true;

    // The normal step
    let step = 1.0 / (n_sqrt) as f64;

    // More complicated step (0,2)
    let mut steps_02 = vec![];
    {
        let mut x_size = n;
        let mut y_size = 1;
        while x_size != 0 {
            println!("{} {}", x_size, y_size);
            steps_02.push((1.0 / (x_size as f64), 1.0 / (y_size as f64)));
            x_size = x_size / 2;
            y_size *= 2;
        }
    }

    let mut functions: Vec<(&str, Box<Generator>, Vec<(f64, f64)>)> = vec![
        ("Uniform", Box::new(Uniform::default()), vec![(step, step)]),
        (
            "Jittered",
            Box::new(Jittered::default()),
            vec![(step, step)],
        ),
        (
            "MultiJittered",
            Box::new(MultiJittered::<HVElementaryElement>::default()),
            vec![
                (step, step),
                (step / n_sqrt as f64, 1.0),
                (1.0, step / n_sqrt as f64),
            ],
        ),
        (
            "MultiJittered (0,2)",
            Box::new(MultiJittered::<ElementaryElement02>::default()),
            steps_02,
        ),
    ];
    for f in &mut functions {
        let (mut x, mut y) = (vec![0.0; n], vec![0.0; n]);
        for i in 0..n {
            let res = f.1.generate();
            x[i] = res.0;
            y[i] = res.1;
        }
        if do_show {
            for s in f.2.iter() {
                show(f.0, &x, &y, *s);
            }
        }
    }
}
