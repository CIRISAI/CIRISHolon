//! READING A QUENCH LOG WITHOUT READING ITS HEADER.
//!
//! `waterquench` prints a header naming the surfaces it loaded:
//!
//! ```text
//! # Physics Path: Pairs (H-H, O-H, O-O) + Complete MBE3 Triples (H3, OH2, O2H, O3)
//! ```
//!
//! `OH2` there is the NAME OF A TABLE — the (O,H,H) three-body surface, listed beside H3,
//! O2H and O3. It is not a molecule, it is not a census entry, and it is the only place
//! the string appears in `conformance/atomworld/p2_waterquench.log`. Every molecule line
//! in that file, across all eight seeds, reads `H2`, `OH`, `O2H`, `O3H3`, `O4H2` or
//! `O4H4`; the run's own headline is **0 of 8 seeds with H₂O as the modal O-containing
//! molecule**.
//!
//! A reader that greps the file for `OH2` therefore finds a hit and concludes water
//! formed. That is not hypothetical — it is how this lane was briefed, and the brief was
//! wrong.
//!
//! So the table names are a POISONED VOCABULARY: the same tokens name surfaces in the
//! header and molecules in the census, and only position tells them apart. This module
//! parses the two apart on purpose. `header_tables` collects the poisoned tokens and keeps
//! them where they can be seen; `molecules` comes only from `seed ` rows, and nothing else
//! in this crate reads a log at all.

/// One `seed ` row of a quench log.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedRow {
    pub seed: u64,
    pub dt: f64,
    /// The modal O-containing molecule, or `None` where the run printed `-`.
    pub modal_o: Option<String>,
    pub free_o: usize,
    pub free_h: usize,
    pub largest: usize,
    /// Compositions from the `molecules [...]` field, and from nowhere else.
    pub molecules: Vec<String>,
    pub fenced: u64,
}

#[derive(Clone, Debug, Default)]
pub struct QuenchLog {
    /// Surface names lifted out of the `Physics Path` header. Collected so that the
    /// poisoned tokens are VISIBLE rather than merely excluded — a vocabulary you cannot
    /// name is one you cannot check yourself against.
    pub header_tables: Vec<String>,
    pub seeds: Vec<SeedRow>,
}

fn field_after<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let i = line.find(key)? + key.len();
    Some(line[i..].trim_start())
}

fn first_token(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

impl QuenchLog {
    pub fn parse(text: &str) -> Self {
        let mut out = QuenchLog::default();
        for line in text.lines() {
            // ---- the header's table names, collected and quarantined ----------------
            if line.starts_with('#') && line.contains("Triples (") {
                if let Some(rest) = line.split("Triples (").nth(1) {
                    if let Some(inner) = rest.split(')').next() {
                        out.header_tables
                            .extend(inner.split(',').map(|t| t.trim().to_string()));
                    }
                }
                continue;
            }
            // Any other comment line is header or verdict prose and carries no census.
            if line.starts_with('#') {
                continue;
            }
            if !line.starts_with("seed ") {
                continue;
            }
            let seed = field_after(line, "seed ")
                .and_then(|s| u64::from_str_radix(first_token(s).trim_start_matches("0x"), 16).ok());
            let dt = field_after(line, " dt ").and_then(|s| first_token(s).parse().ok());
            let (Some(seed), Some(dt)) = (seed, dt) else {
                continue;
            };
            let modal = field_after(line, "modal-O ").map(first_token).unwrap_or("-");
            let molecules: Vec<String> = line
                .find("molecules [")
                .and_then(|i| {
                    let rest = &line[i + "molecules [".len()..];
                    rest.find(']').map(|j| &rest[..j])
                })
                .map(|inner| {
                    inner
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            out.seeds.push(SeedRow {
                seed,
                dt,
                modal_o: if modal == "-" { None } else { Some(modal.to_string()) },
                free_o: field_after(line, "free O ")
                    .and_then(|s| first_token(s).parse().ok())
                    .unwrap_or(0),
                free_h: field_after(line, "free H ")
                    .and_then(|s| first_token(s).parse().ok())
                    .unwrap_or(0),
                largest: field_after(line, "largest ")
                    .and_then(|s| first_token(s).parse().ok())
                    .unwrap_or(0),
                molecules,
                fenced: field_after(line, "fenced ")
                    .and_then(|s| first_token(s).parse().ok())
                    .unwrap_or(0),
            });
        }
        out
    }

    /// Every molecule across every seed, with its count. Sorted for a stable comparison.
    pub fn molecule_census(&self) -> Vec<(String, usize)> {
        let mut out: Vec<(String, usize)> = Vec::new();
        for s in &self.seeds {
            for m in &s.molecules {
                match out.iter_mut().find(|(f, _)| f == m) {
                    Some(slot) => slot.1 += 1,
                    None => out.push((m.clone(), 1)),
                }
            }
        }
        out.sort();
        out
    }

    /// How many times a composition appears as an actual MOLECULE. The question a reader
    /// should be asking, and the one a grep of the raw text does not answer.
    pub fn molecule_count(&self, formula: &str) -> usize {
        self.seeds
            .iter()
            .flat_map(|s| s.molecules.iter())
            .filter(|m| *m == formula)
            .count()
    }

    /// Whether a token appears in the header's surface list — i.e. whether it is one of
    /// the poisoned ones.
    pub fn is_header_table(&self, token: &str) -> bool {
        self.header_tables.iter().any(|t| t == token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE PRIMARY ARTIFACT, not a copy of it. If the file moves this test fails loudly,
    /// which is the correct behaviour for a gate that exists to bind a reading to a
    /// specific banked run.
    const P2: &str = include_str!("../../../../conformance/atomworld/p2_waterquench.log");

    /// THE PLANT the lead asked for, in its sharpest form: the failure mode demonstrated
    /// beside the correct reading, on the real file.
    ///
    /// A grep of the raw text finds `OH2`. The census finds none. Both facts are asserted
    /// here so that the gate cannot pass by the string simply being absent — the string IS
    /// present, in the header, and the parser has to be the thing that separates them.
    #[test]
    fn a_header_table_name_is_never_a_molecule() {
        // The failure mode, reproduced.
        let naive_hits = P2.lines().filter(|l| l.contains("OH2")).count();
        assert_eq!(naive_hits, 1, "a grep for OH2 DOES hit this file");
        let hit = P2.lines().find(|l| l.contains("OH2")).unwrap();
        assert!(hit.starts_with('#'), "and its one hit is a comment: {hit}");
        assert!(hit.contains("Triples ("), "specifically the surface list");

        // The correct reading.
        let log = QuenchLog::parse(P2);
        assert!(log.is_header_table("OH2"), "OH2 is a registered SURFACE");
        assert_eq!(
            log.molecule_count("OH2"),
            0,
            "and it is not a molecule in any seed of this run"
        );
        assert_eq!(log.molecule_count("H2O"), 0, "nor under the other spelling");
    }

    /// A log that registers surfaces and produces NOTHING must census empty — the lead's
    /// stated form of the plant, on a synthetic log so the emptiness is by construction.
    #[test]
    fn tables_registered_and_zero_molecules_yields_an_empty_census() {
        let synthetic = "\
# Physics Path: Pairs (H-H, O-H, O-O) + Complete MBE3 Triples (H3, OH2, O2H, O3)
# arm = mixed (8 H + 4 O)   seeds = 8
# molecule census over all seeds: (none)
";
        let log = QuenchLog::parse(synthetic);
        assert_eq!(log.header_tables, vec!["H3", "OH2", "O2H", "O3"]);
        assert!(log.seeds.is_empty());
        assert!(
            log.molecule_census().is_empty(),
            "surfaces registered is not molecules formed"
        );
        assert_eq!(log.molecule_count("OH2"), 0);
        // And the failure mode is live on this fixture too.
        assert!(synthetic.contains("OH2"));
    }

    /// The parser reproduces the log's OWN aggregate line. A parser that disagrees with
    /// the run it is parsing is not a reading of that run.
    #[test]
    fn the_parsed_census_reproduces_the_logs_own_census_line() {
        let log = QuenchLog::parse(P2);
        assert_eq!(log.seeds.len(), 8, "eight staked seeds");
        let mine = log.molecule_census();
        // The file's own line: "19xH2  1xOH  4xO2H  1xO3H3  1xO4H2  4xO4H4"
        let theirs: Vec<(String, usize)> = P2
            .lines()
            .find(|l| l.starts_with("# molecule census over all seeds:"))
            .expect("the log states its own census")
            .split(':')
            .nth(1)
            .unwrap()
            .split_whitespace()
            .map(|tok| {
                let (n, f) = tok.split_once('x').expect("NxFORMULA");
                (f.to_string(), n.parse::<usize>().unwrap())
            })
            .collect();
        let mut theirs = theirs;
        theirs.sort();
        assert_eq!(mine, theirs, "parsed census must equal the run's own");
        // And the shape of that census, spelled out, because it is the actual result:
        assert_eq!(log.molecule_count("H2"), 19);
        assert_eq!(log.molecule_count("OH"), 1);
        assert_eq!(log.molecule_count("O4H4"), 4);
    }

    /// The run's headline, asserted rather than remembered: no seed made water.
    #[test]
    fn no_seed_of_the_banked_run_produced_water() {
        let log = QuenchLog::parse(P2);
        let water = log
            .seeds
            .iter()
            .filter(|s| s.modal_o.as_deref() == Some("OH2"))
            .count();
        assert_eq!(water, 0, "0 of 8, which is the run's own verdict");
        // Seed 0x...5422 -- "seed 2" in the briefing that started this lane -- made OH.
        let s2 = log
            .seeds
            .iter()
            .find(|s| s.seed == 0x0000_0000_5341_5422)
            .unwrap();
        assert_eq!(s2.modal_o.as_deref(), Some("OH"));
        assert_eq!(s2.molecules, vec!["H2", "H2", "OH", "O3H3"]);
        assert_eq!(s2.fenced, 0, "this run served all four surfaces");
    }

    #[test]
    fn seed_rows_parse_every_field() {
        let log = QuenchLog::parse(P2);
        let first = &log.seeds[0];
        assert_eq!(first.seed, 0x0000_0000_5341_5421);
        assert_eq!(first.dt, 0.5386);
        assert_eq!(first.modal_o.as_deref(), Some("O4H4"));
        assert_eq!(first.free_o, 0);
        assert_eq!(first.free_h, 0);
        assert_eq!(first.largest, 8);
        assert_eq!(first.molecules, vec!["H2", "H2", "O4H4"]);
    }
}
