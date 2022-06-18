use criterion::{criterion_group, criterion_main, Criterion};

use sample::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("play game", |b| {
        b.iter(|| {
            let mut the_goose = SimpleTheGoose::new((1u32..7u32).cycle());

            let players = vec![SamplePlayer("Pippo"), SamplePlayer("Pluto"), SamplePlayer("Paperino")];
            for player in &players {
                the_goose
                    .execute(Command::Add(*player))
                    .expect("Adding player");
            }

            'outher: loop {
                for player in &players {
                    if the_goose
                        .execute(Command::RollAndMove(*player))
                        .unwrap()
                        .iter()
                        .any(|event| match event {
                            Event::Win(_) => true,
                            _ => false,
                        })
                    {
                        break 'outher;
                    }
                }
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
