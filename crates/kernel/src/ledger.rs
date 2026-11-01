//! Ledgers: double-entry accounts of a conserved quantity. Every transfer debits one
//! account and credits another, so the sum over all accounts is zero by construction.
//! The check that matters is different: a module that holds a stock must declare it,
//! and at every close the declared stock is compared with what the flows imply. The
//! difference is a leak, and a leak above tolerance is a failed close.

use crate::time::Time;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Close {
    pub t: Time,
    pub ok: bool,
    pub max_leak: f64,
    /// declared minus flow-derived, per stock account
    pub leaks: BTreeMap<String, f64>,
    pub balances: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub name: String,
    pub unit: String,
    pub tolerance: f64,
    /// Flow-derived balance of every account, boundaries included.
    pub balances: BTreeMap<String, f64>,
    /// Latest declared value of each stock account.
    pub declared: BTreeMap<String, f64>,
    pub stocks: BTreeSet<String>,
    pub transfers: u64,
    pub closes: Vec<Close>,
}

impl Ledger {
    pub fn new(name: &str, unit: &str, tolerance: f64) -> Ledger {
        Ledger {
            name: name.to_string(), unit: unit.to_string(), tolerance,
            balances: BTreeMap::new(), declared: BTreeMap::new(), stocks: BTreeSet::new(),
            transfers: 0, closes: Vec::new(),
        }
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: f64) {
        debug_assert!(amount.is_finite(), "non-finite transfer on ledger {}", self.name);
        *self.balances.entry(from.to_string()).or_insert(0.0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0.0) += amount;
        self.transfers += 1;
    }

    /// A module states how much of the quantity it actually holds. The first declaration
    /// of an account is its opening balance; later ones are checked against the flows.
    pub fn declare_stock(&mut self, account: &str, value: f64) {
        if self.stocks.insert(account.to_string()) {
            // opening balance: credit from the "opening" boundary so double entry holds
            let delta = value - self.balances.get(account).copied().unwrap_or(0.0);
            *self.balances.entry("opening".to_string()).or_insert(0.0) -= delta;
            *self.balances.entry(account.to_string()).or_insert(0.0) += delta;
        }
        self.declared.insert(account.to_string(), value);
    }

    pub fn close(&mut self, t: Time) -> &Close {
        let mut leaks = BTreeMap::new();
        let mut max_leak = 0.0f64;
        for acct in &self.stocks {
            let derived = self.balances.get(acct).copied().unwrap_or(0.0);
            let declared = self.declared.get(acct).copied().unwrap_or(0.0);
            let leak = declared - derived;
            max_leak = max_leak.max(leak.abs());
            leaks.insert(acct.clone(), leak);
        }
        self.closes.push(Close { t, ok: max_leak <= self.tolerance, max_leak, leaks, balances: self.balances.clone() });
        self.closes.last().unwrap()
    }

    pub fn all_closed(&self) -> bool { self.closes.iter().all(|c| c.ok) }
}

/// What a run writes out for a ledger.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerReport {
    pub name: String,
    pub unit: String,
    pub tolerance: f64,
    pub transfers: u64,
    pub accounts: Vec<String>,
    pub stocks: Vec<String>,
    pub closes: Vec<Close>,
    pub all_closed: bool,
}

impl From<&Ledger> for LedgerReport {
    fn from(l: &Ledger) -> Self {
        LedgerReport {
            name: l.name.clone(), unit: l.unit.clone(), tolerance: l.tolerance, transfers: l.transfers,
            accounts: l.balances.keys().cloned().collect(), stocks: l.stocks.iter().cloned().collect(),
            closes: l.closes.clone(), all_closed: l.all_closed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leak_is_caught() {
        let mut l = Ledger::new("water", "m3", 1e-9);
        l.declare_stock("bucket", 10.0);
        l.transfer("sky", "bucket", 5.0);
        l.declare_stock("bucket", 15.0);
        assert!(l.close(Time(0)).ok);
        l.transfer("bucket", "sea", 2.0);
        l.declare_stock("bucket", 12.5); // module thinks it lost 2.5 but moved 2
        assert!(!l.close(Time(1)).ok);
        assert!((l.closes[1].leaks["bucket"] + 0.5).abs() < 1e-12);
    }
}
