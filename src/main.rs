extern crate ordered_float;

use std::fmt;
use std::io::{self, BufRead, BufReader, Read};
use ordered_float::OrderedFloat;

// Basic ELO algorithm: R += K(S - Q/(Q + Q_opp))
//                where Q = 10 ** (R / 400)

struct TeamResult {
    rating: f64,
    points: f64, // This can be any norm, we use the fraction.
}

fn elo_adjustment(team: &TeamResult, opponent: &TeamResult) -> f64 {
    const K: f64 = 10.0;

    if team.points < 0.0 || opponent.points < 0.0 {
        panic!("Points must be nonnegative");
    }
    if team.points == 0.0 && opponent.points == 0.0 {
        return 0.0;
    }
    let q = |rating| 10.0f64.powf(rating / 400.0);
    let expected = q(team.rating) / (q(team.rating) + q(opponent.rating));
    let actual = team.points / (team.points + opponent.points);
    K * (actual - expected)
}

// Batch ELO: compute total adjustment from a bunch of "simultaneous" games.

#[derive(Clone)]
struct Game {
    index_a: usize,
    index_b: usize,
    points_a: f64,
    points_b: f64,
}

fn batch_adjustments(ratings: &Vec<f64>, games: &Vec<Game>) -> Vec<f64> {
    let mut adjustments = vec![0.0; ratings.len()];
    for game in games {
        let a = TeamResult {
            rating: ratings[game.index_a],
            points: game.points_a,
        };
        let b = TeamResult {
            rating: ratings[game.index_b],
            points: game.points_b,
        };
        adjustments[game.index_a] += elo_adjustment(&a, &b);
        adjustments[game.index_b] += elo_adjustment(&b, &a);
    }
    adjustments
}

// Iterated batch ELO: starting from 1500 rating, apply batch ELO until convergence.

#[derive(Debug)]
struct ConvergenceFailure {
    rounds: usize,
    last_ratings: Vec<f64>,
    last_adjustments: Vec<f64>,
}

impl fmt::Display for ConvergenceFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "No convergence after {} rounds.\nLast ratings: {:?}\nLast adjustments: {:?}",
               self.rounds,
               self.last_ratings,
               self.last_adjustments)
    }
}

impl std::error::Error for ConvergenceFailure {
    fn description(&self) -> &str {
        "No convergence"
    }
}

fn elo_ratings(n_teams: usize, games: &Vec<Game>) -> Result<Vec<f64>, ConvergenceFailure> {
    const EPSILON: f64 = 0.01;
    const LIMIT: usize = 10000;
    let mut ratings = vec![1500.0; n_teams];
    let mut adjustments = vec![0.0; n_teams];
    for _ in 0..LIMIT {
        adjustments = batch_adjustments(&ratings, games);
        let mut converged = true;
        for (i, a) in adjustments.iter().enumerate() {
            ratings[i] += *a;
            if a.abs() > EPSILON {
                converged = false;
            }
        }
        if converged {
            return Ok(ratings);
        }
    }
    Err(ConvergenceFailure {
        rounds: LIMIT,
        last_ratings: ratings,
        last_adjustments: adjustments,
    })
}

// Read competition data from CSV file

struct Competition {
    teams: Vec<String>,
    games: Vec<Game>,
}

fn read_csv(file: &mut Read) -> Competition {
    let mut teams = Vec::new();
    let mut games = Vec::new();
    let find_or_add = |teams: &mut Vec<String>, name| {
        match teams.iter().position(|n| name == *n) {
            Some(pos) => pos,
            None => {
                teams.push(name);
                teams.len() - 1
            }
        }
    };
    let read = BufReader::new(file);
    for l in read.lines() {
        let line = l.unwrap();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        let parts: Vec<_> = line.split(',').collect();
        if parts.len() != 4 {
            panic!("Malformed line: {}", line)
        }
        games.push(Game {
            index_a: find_or_add(&mut teams, parts[0].to_string()),
            index_b: find_or_add(&mut teams, parts[1].to_string()),
            points_a: parts[2].parse().unwrap(),
            points_b: parts[3].parse().unwrap(),
        });
    }
    Competition {
        teams: teams,
        games: games,
    }
}

// Read competition data from stdin, compute fixed-point ratings, print them.
fn main() {
    let comp = read_csv(&mut io::stdin());
    let ratings = elo_ratings(comp.teams.len(), &comp.games).unwrap();
    let mut rated_teams: Vec<_> = comp.teams.iter().zip(ratings).collect();
    rated_teams.sort_by_key(|&(_, rating)| OrderedFloat(-rating));
    for (team, rating) in rated_teams {
        println!("{}: {}", team, rating as i32)
    }
}
