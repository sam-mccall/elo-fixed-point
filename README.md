# elo-fixed-point
Calculates Elo ratings for team strength based on an unordered set of games.

The [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system) is normally used to find the *current* strength of a team (or player), based on history against players that are already rated.

Here, we use it to find the *average* strength of each team over a set of games (e.g. a football season), with no prior rankings. This is more accurate than just counting wins if not all teams play the same opponents (differing strength of schedule).

# Method

Initially each team is assigned a rating of 1500. Then for each team, we run through each of their games and compute the Elo score adjustment, *without adjusting scores between games*.

These adjustments are then applied as a batch, and the process is repeated. After the first round, the ratings just reflect the amount of times a team wins, but subsequent rounds incorporate strength of schedule too.

We stop when the scores converge: when no team had an adjustment of more than 0.1 Elo points over the season.

# Scoring

Internally we care about the "points fraction" that a team got in a game, which will depend on your scoring system.

E.g. if team A beats team B and the match score of 3-1 is entered, then we consider A to be "75% winning" and B to be "25% winning".
If you prefer simple win/loss-based calculations, then enter scores of 1 and 0 instead. You could also use tournament points etc.

The system most appropriate probably depends on the dynamics of the sport.

# Usage

    cargo run < data/sr2016.csv

Input format is simple CSV, with columns: Team 1, Team 2, Team 1 score, Team 2 score, e.g.

    Blues,Highlanders,33,31
    Brumbies,Hurricanes,52,10
    Brumbies,Waratahs,32,15
    Highlanders,Hurricanes,17,16

See data/ directory for full example.

# Caveats/possible expansion

  - Each game is weighted the same.
  - No theoretical basis for any of this, though it does seem to converge :-)
  - It'd be nice to plot ratings after each rescoring round to show convergence.
