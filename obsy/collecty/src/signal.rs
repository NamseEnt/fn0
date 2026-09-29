use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Signal {
    Logs,
    Traces,
    Metrics,
}

impl Signal {
    pub const ALL: [Signal; 3] = [Signal::Logs, Signal::Traces, Signal::Metrics];

    pub fn as_str(self) -> &'static str {
        match self {
            Signal::Logs => "logs",
            Signal::Traces => "traces",
            Signal::Metrics => "metrics",
        }
    }

    /// The OTLP/HTTP path this signal is exported to.
    pub fn otlp_path(self) -> &'static str {
        match self {
            Signal::Logs => "/v1/logs",
            Signal::Traces => "/v1/traces",
            Signal::Metrics => "/v1/metrics",
        }
    }

    pub fn from_otlp_path(path: &str) -> Option<Signal> {
        Signal::ALL
            .into_iter()
            .find(|signal| signal.otlp_path() == path)
    }

    /// Which of the three queues this signal owns.
    ///
    /// A slot in memory and nothing else. The signal is named on the wire and
    /// in the queue's directory names, so there is no number anywhere for this
    /// to have to agree with.
    pub fn index(self) -> usize {
        match self {
            Signal::Logs => 0,
            Signal::Traces => 1,
            Signal::Metrics => 2,
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_signal_round_trips_through_its_otlp_path() {
        for signal in Signal::ALL {
            assert_eq!(Signal::from_otlp_path(signal.otlp_path()), Some(signal));
        }
        assert_eq!(Signal::from_otlp_path("/v1/profiles"), None);
    }

    #[test]
    fn every_signal_has_a_slot_of_its_own() {
        for (slot, signal) in Signal::ALL.into_iter().enumerate() {
            assert_eq!(signal.index(), slot);
        }
    }
}
