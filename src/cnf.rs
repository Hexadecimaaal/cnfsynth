use std::fmt::Display;
use std::str::FromStr;

pub type Lit = isize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub lits: Vec<Lit>,
}

impl From<Vec<Lit>> for Clause {
    fn from(lits: Vec<Lit>) -> Self {
        Clause { lits }
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, lit) in self.lits.iter().enumerate() {
            write!(f, "{} ", lit)?;
        }
        write!(f, "0")?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cnf {
    pub clauses: Vec<Clause>,
    pub top: Lit,
}

impl From<Vec<Clause>> for Cnf {
    fn from(clauses: Vec<Clause>) -> Self {
        let top = clauses.iter().flat_map(|c| c.lits.iter()).map(|lit| lit.abs()).max().unwrap_or(1);
        Cnf {
            clauses,
            top,
        }
    }
}

impl Display for Cnf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "p cnf {} {}\n", self.top - 1, self.clauses.len())?;
        for clause in self.clauses.iter() {
            write!(f, "{}\n", clause)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sat(pub Option<Vec<Lit>>);

impl FromStr for Sat {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines();
        let mut lines = lines.skip_while(|line| !line.starts_with("s "));
        let s = lines.next().unwrap();
        if s.starts_with("s UNSAT") {
            Ok(Sat(None))
        } else if s.starts_with("s SAT") {
            let mut result = vec![];
            let vars = lines.filter(|line| line.starts_with("v ")).flat_map(|line| line[2..].split_whitespace());
            for v in vars {
                let v: Lit = v.parse().unwrap();
                if v == 0 {
                    break;
                }
                result.resize(v.abs() as usize + 1, 0);
                result[v.abs() as usize] = v;
            }
            Ok(Sat(Some(result)))
        } else {
            Err(())
        }
    }
}
