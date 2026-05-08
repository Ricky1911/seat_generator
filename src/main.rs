mod generator;

use generator::{Generator, SeatingChart, Zone};
use std::collections::HashMap;

fn main() {
    let capacities = HashMap::from([
        (Zone::Zone1, 2),
        (Zone::Zone2a, 2),
        (Zone::Zone2b, 2),
        (Zone::Zone3, 4),
    ]);

    let g = Generator::new(capacities);

    let chart_recent = SeatingChart::new(HashMap::from([
        (0, Zone::Zone1),
        (1, Zone::Zone1),
        (2, Zone::Zone2a),
        (3, Zone::Zone2a),
        (4, Zone::Zone2b),
        (5, Zone::Zone2b),
        (6, Zone::Zone3),
        (7, Zone::Zone3),
        (8, Zone::Zone3),
        (9, Zone::Zone3),
    ]));

    let chart_previous = SeatingChart::new(HashMap::from([
        (0, Zone::Zone3),
        (1, Zone::Zone3),
        (2, Zone::Zone3),
        (3, Zone::Zone3),
        (4, Zone::Zone3),
        (5, Zone::Zone3),
        (6, Zone::Zone1),
        (7, Zone::Zone2a),
        (8, Zone::Zone2b),
        (9, Zone::Zone3),
    ]));

    match g.generate(&chart_recent, &chart_previous) {
        Ok(chart) => {
            println!("Generated seating chart:");
            let mut people: Vec<_> = chart.assignments.iter().collect();
            people.sort_by_key(|(id, _)| *id);
            for (id, zone) in people {
                println!("  Person {}: {:?}", id, zone);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
