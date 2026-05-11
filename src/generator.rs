use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Zone1,
    Zone2a,
    Zone2b,
    Zone3,
}

impl Zone {
    #[allow(dead_code)]
    pub fn all() -> [Zone; 4] {
        [Zone::Zone1, Zone::Zone2a, Zone::Zone2b, Zone::Zone3]
    }
}

pub type PersonId = String;

#[derive(Debug, Clone)]
pub struct SeatingChart {
    pub assignments: HashMap<PersonId, Zone>,
}

impl SeatingChart {
    pub fn new(assignments: HashMap<PersonId, Zone>) -> Self {
        Self { assignments }
    }

    pub fn get(&self, person: &str) -> Option<Zone> {
        self.assignments.get(person).copied()
    }

    #[allow(dead_code)]
    pub fn people(&self) -> Vec<PersonId> {
        let mut people: Vec<PersonId> = self.assignments.keys().cloned().collect();
        people.sort();
        people
    }
}

#[derive(Debug, Clone)]
pub struct Generator {
    capacities: HashMap<Zone, usize>,
}

impl Generator {
    pub fn new(capacities: HashMap<Zone, usize>) -> Self {
        Self { capacities }
    }

    fn allowed_zones(&self, prev1: Option<Zone>, prev2: Option<Zone>) -> Vec<Zone> {
        let mut allowed = vec![Zone::Zone3];

        let Some(prev1_zone) = prev1 else {
            allowed.extend([Zone::Zone1, Zone::Zone2a, Zone::Zone2b]);
            return allowed;
        };

        if prev1_zone != Zone::Zone1 {
            allowed.push(Zone::Zone1);
        }
        if prev1_zone != Zone::Zone2a {
            allowed.push(Zone::Zone2a);
        }
        if prev1_zone != Zone::Zone2b {
            allowed.push(Zone::Zone2b);
        }

        if let Some(prev2_zone) = prev2 {
            let prev1_in_2 = matches!(prev1_zone, Zone::Zone2a | Zone::Zone2b);
            let prev2_in_2 = matches!(prev2_zone, Zone::Zone2a | Zone::Zone2b);
            if prev1_in_2 && prev2_in_2 {
                allowed.retain(|z| !matches!(z, Zone::Zone2a | Zone::Zone2b));
            }
        }

        allowed
    }

    fn try_assign(
        &self,
        person_options: &[(PersonId, Vec<Zone>)],
    ) -> Option<HashMap<PersonId, Zone>> {
        let mut rng = rand::rng();
        let mut remaining = self.capacities.clone();
        let mut result = HashMap::new();

        let mut indices: Vec<usize> = (0..person_options.len()).collect();
        indices.shuffle(&mut rng);

        for &idx in &indices {
            let (person, allowed) = &person_options[idx];
            let available: Vec<Zone> = allowed
                .iter()
                .filter(|&z| remaining.get(z).copied().unwrap_or(0) > 0)
                .copied()
                .collect();

            if available.is_empty() {
                return None;
            }

            let chosen = available.choose(&mut rng).unwrap();
            *remaining.get_mut(chosen).unwrap() -= 1;
            result.insert(person.clone(), *chosen);
        }

        Some(result)
    }

    pub fn generate(
        &self,
        chart_most_recent: &SeatingChart,
        chart_previous: &SeatingChart,
    ) -> Result<SeatingChart, String> {
        println!("{}", chart_previous.assignments.keys().len());
        println!("{}", chart_most_recent.assignments.keys().len());
        let all_people = {
            let mut people: Vec<PersonId> = chart_most_recent
                .assignments
                .keys()
                .chain(chart_previous.assignments.keys())
                .cloned()
                .collect();
            people.sort();
            people.dedup();
            people
        };

        if all_people.is_empty() {
            return Err("no people to assign".to_string());
        }

        let total_capacity: usize = self.capacities.values().sum();
        if all_people.len() != total_capacity {
            return Err(format!(
                "people count ({}) does not match total capacity ({})",
                all_people.len(),
                total_capacity
            ));
        }

        let person_options: Vec<(PersonId, Vec<Zone>)> = all_people
            .iter()
            .map(|p| {
                let prev1 = chart_most_recent.get(p);
                let prev2 = chart_previous.get(p);
                (p.clone(), self.allowed_zones(prev1, prev2))
            })
            .collect();

        for (person, allowed) in &person_options {
            if allowed.is_empty() {
                return Err(format!("person {} has no allowed zones", person));
            }
        }

        for _ in 0..200 {
            if let Some(assignments) = self.try_assign(&person_options) {
                return Ok(SeatingChart::new(assignments));
            }
        }

        Err("failed to generate a valid seating chart after max attempts".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> PersonId {
        s.to_string()
    }

    fn make_chart(data: Vec<(&str, Zone)>) -> SeatingChart {
        SeatingChart::new(data.into_iter().map(|(n, z)| (id(n), z)).collect())
    }

    fn default_capacities() -> HashMap<Zone, usize> {
        HashMap::from([
            (Zone::Zone1, 2),
            (Zone::Zone2a, 2),
            (Zone::Zone2b, 2),
            (Zone::Zone3, 4),
        ])
    }

    #[test]
    fn test_empty_charts() {
        let g = Generator::new(default_capacities());
        let empty = SeatingChart::new(HashMap::new());
        assert!(g.generate(&empty, &empty).is_err());
    }

    #[test]
    fn test_capacity_mismatch() {
        let g = Generator::new(HashMap::from([(Zone::Zone3, 1)]));
        let chart = make_chart(vec![("p0", Zone::Zone3), ("p1", Zone::Zone3)]);
        assert!(g.generate(&chart, &chart).is_err());
    }

    #[test]
    fn test_no_solution_possible() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 1),
            (Zone::Zone2a, 0),
            (Zone::Zone2b, 0),
            (Zone::Zone3, 0),
        ]));
        let recent = make_chart(vec![("p0", Zone::Zone1)]);
        let prev = make_chart(vec![("p0", Zone::Zone3)]);
        assert!(g.generate(&recent, &prev).is_err());
    }

    #[test]
    fn test_zone1_no_consecutive() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 1),
            (Zone::Zone2a, 0),
            (Zone::Zone2b, 0),
            (Zone::Zone3, 1),
        ]));
        let recent = make_chart(vec![("p0", Zone::Zone1), ("p1", Zone::Zone3)]);
        let prev = make_chart(vec![("p0", Zone::Zone3), ("p1", Zone::Zone3)]);
        let result = g.generate(&recent, &prev).unwrap();
        assert_ne!(result.get("p0"), Some(Zone::Zone1));
    }

    #[test]
    fn test_zone2_individual_no_consecutive() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 0),
            (Zone::Zone2a, 1),
            (Zone::Zone2b, 1),
            (Zone::Zone3, 0),
        ]));
        let recent = make_chart(vec![("p0", Zone::Zone2a), ("p1", Zone::Zone2b)]);
        let prev = make_chart(vec![("p0", Zone::Zone3), ("p1", Zone::Zone3)]);
        let result = g.generate(&recent, &prev).unwrap();
        assert_ne!(result.get("p0"), Some(Zone::Zone2a));
        assert_ne!(result.get("p1"), Some(Zone::Zone2b));
    }

    #[test]
    fn test_zone2_combined_two_consecutive() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 1),
            (Zone::Zone2a, 0),
            (Zone::Zone2b, 0),
            (Zone::Zone3, 0),
        ]));
        let recent = make_chart(vec![("p0", Zone::Zone2b)]);
        let prev = make_chart(vec![("p0", Zone::Zone2a)]);
        let result = g.generate(&recent, &prev).unwrap();
        assert!(!matches!(
            result.get("p0"),
            Some(Zone::Zone2a | Zone::Zone2b)
        ));
    }

    #[test]
    fn test_zone2_one_consecutive_allowed() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 0),
            (Zone::Zone2a, 1),
            (Zone::Zone2b, 1),
            (Zone::Zone3, 0),
        ]));
        let recent = make_chart(vec![("p0", Zone::Zone2a), ("p1", Zone::Zone2b)]);
        let prev = make_chart(vec![("p0", Zone::Zone3), ("p1", Zone::Zone3)]);
        let result = g.generate(&recent, &prev).unwrap();
        assert_ne!(result.get("p0"), Some(Zone::Zone2a));
        assert_ne!(result.get("p1"), Some(Zone::Zone2b));
    }

    #[test]
    fn test_zone3_always_allowed() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 0),
            (Zone::Zone2a, 0),
            (Zone::Zone2b, 0),
            (Zone::Zone3, 1),
        ]));
        let recent = make_chart(vec![("p0", Zone::Zone3)]);
        let prev = make_chart(vec![("p0", Zone::Zone3)]);
        let result = g.generate(&recent, &prev).unwrap();
        assert_eq!(result.get("p0"), Some(Zone::Zone3));
    }

    #[test]
    fn test_full_generation_is_valid() {
        let g = Generator::new(default_capacities());

        let recent = make_chart(vec![
            ("p0", Zone::Zone1),
            ("p1", Zone::Zone1),
            ("p2", Zone::Zone2a),
            ("p3", Zone::Zone2a),
            ("p4", Zone::Zone2b),
            ("p5", Zone::Zone2b),
            ("p6", Zone::Zone3),
            ("p7", Zone::Zone3),
            ("p8", Zone::Zone3),
            ("p9", Zone::Zone3),
        ]);

        let prev = make_chart(vec![
            ("p0", Zone::Zone3),
            ("p1", Zone::Zone3),
            ("p2", Zone::Zone3),
            ("p3", Zone::Zone3),
            ("p4", Zone::Zone3),
            ("p5", Zone::Zone3),
            ("p6", Zone::Zone1),
            ("p7", Zone::Zone2a),
            ("p8", Zone::Zone2b),
            ("p9", Zone::Zone3),
        ]);

        let result = g.generate(&recent, &prev).unwrap();

        assert_eq!(result.assignments.len(), 10);

        let mut zone_counts = HashMap::new();
        for zone in result.assignments.values() {
            *zone_counts.entry(*zone).or_insert(0) += 1;
        }
        assert_eq!(zone_counts.get(&Zone::Zone1).copied().unwrap_or(0), 2);
        assert_eq!(zone_counts.get(&Zone::Zone2a).copied().unwrap_or(0), 2);
        assert_eq!(zone_counts.get(&Zone::Zone2b).copied().unwrap_or(0), 2);
        assert_eq!(zone_counts.get(&Zone::Zone3).copied().unwrap_or(0), 4);

        for (person, zone) in &result.assignments {
            let prev1 = recent.get(person);
            let prev2 = prev.get(person);

            match zone {
                Zone::Zone1 => {
                    assert_ne!(
                        prev1,
                        Some(Zone::Zone1),
                        "person {} violated Zone1 consecutive rule",
                        person
                    );
                }
                Zone::Zone2a => {
                    assert_ne!(
                        prev1,
                        Some(Zone::Zone2a),
                        "person {} violated Zone2a consecutive rule",
                        person
                    );
                    if let (Some(p1), Some(p2)) = (prev1, prev2) {
                        let p1_in_2 = matches!(p1, Zone::Zone2a | Zone::Zone2b);
                        let p2_in_2 = matches!(p2, Zone::Zone2a | Zone::Zone2b);
                        assert!(
                            !(p1_in_2 && p2_in_2),
                            "person {} violated combined Zone2 consecutive rule",
                            person
                        );
                    }
                }
                Zone::Zone2b => {
                    assert_ne!(
                        prev1,
                        Some(Zone::Zone2b),
                        "person {} violated Zone2b consecutive rule",
                        person
                    );
                    if let (Some(p1), Some(p2)) = (prev1, prev2) {
                        let p1_in_2 = matches!(p1, Zone::Zone2a | Zone::Zone2b);
                        let p2_in_2 = matches!(p2, Zone::Zone2a | Zone::Zone2b);
                        assert!(
                            !(p1_in_2 && p2_in_2),
                            "person {} violated combined Zone2 consecutive rule",
                            person
                        );
                    }
                }
                Zone::Zone3 => {}
            }
        }
    }

    #[test]
    fn test_new_person_in_previous_only() {
        let g = Generator::new(HashMap::from([
            (Zone::Zone1, 1),
            (Zone::Zone2a, 0),
            (Zone::Zone2b, 0),
            (Zone::Zone3, 0),
        ]));
        let recent = make_chart(vec![]);
        let prev = make_chart(vec![("p0", Zone::Zone1)]);
        let result = g.generate(&recent, &prev).unwrap();
        assert_eq!(result.get("p0"), Some(Zone::Zone1));
    }
}
