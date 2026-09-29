#[derive(Debug, Clone)]
pub enum LineFilter {
    Contains(String),
    NotContains(String),
    Regex(Regex),
    NotRegex(Regex),
}

impl LineFilter {
    pub fn matches(&self, line: &str) -> bool {
        match self {
            Self::Contains(s) => line.contains(s),
            Self::NotContains(s) => !line.contains(s),
            Self::Regex(re) => re.is_match(line),
            Self::NotRegex(re) => !re.is_match(line),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldOp {
    Eq,
    Neq,
    Regex,
    NotRegex,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Regex(Regex),
    Number(Decimal),
    Duration(i64),
}

/// An exact, bounded decimal representation for field comparisons.
///
/// `f64` is intentionally not used here: values above 2^53 must retain their
/// distinct integer representations, and NaN/Inf must never participate in a
/// field predicate. The coefficient is bounded to i128 and the scale is
/// bounded to keep malformed input from causing unbounded allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decimal {
    coefficient: i128,
    scale: i32,
}

const MAX_DECIMAL_SCALE: i64 = 1_024;

impl Decimal {
    pub(crate) fn parse(input: &str) -> Result<Self, ()> {
        let input = input.trim();
        if input.is_empty() {
            return Err(());
        }

        let (mantissa, exponent) = match input.find(['e', 'E']) {
            Some(index) => {
                if input[index + 1..].contains(['e', 'E']) {
                    return Err(());
                }
                let exponent = input[index + 1..].parse::<i64>().map_err(|_| ())?;
                (&input[..index], exponent)
            }
            None => (input, 0),
        };
        let (negative, mantissa) = if let Some(value) = mantissa.strip_prefix('-') {
            (true, value)
        } else if let Some(value) = mantissa.strip_prefix('+') {
            (false, value)
        } else {
            (false, mantissa)
        };

        let (integer, fraction) = match mantissa.split_once('.') {
            Some((integer, fraction)) if !fraction.contains('.') => (integer, fraction),
            Some(_) => return Err(()),
            None => (mantissa, ""),
        };
        if integer.is_empty() && fraction.is_empty() {
            return Err(());
        }
        if !integer.bytes().all(|byte| byte.is_ascii_digit())
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(());
        }

        let digits = format!("{integer}{fraction}");
        let digits = digits.trim_start_matches('0');
        if digits.is_empty() {
            return Ok(Self {
                coefficient: 0,
                scale: 0,
            });
        }
        let magnitude = digits.parse::<u128>().map_err(|_| ())?;
        let mut coefficient = if negative {
            if magnitude == i128::MAX as u128 + 1 {
                i128::MIN
            } else {
                -i128::try_from(magnitude).map_err(|_| ())?
            }
        } else {
            i128::try_from(magnitude).map_err(|_| ())?
        };
        let mut scale = i64::try_from(fraction.len())
            .map_err(|_| ())?
            .checked_sub(exponent)
            .ok_or(())?;
        if !(-MAX_DECIMAL_SCALE..=MAX_DECIMAL_SCALE).contains(&scale) {
            return Err(());
        }
        while coefficient % 10 == 0 {
            coefficient /= 10;
            scale = scale.checked_sub(1).ok_or(())?;
        }
        if !(-MAX_DECIMAL_SCALE..=MAX_DECIMAL_SCALE).contains(&scale) {
            return Err(());
        }
        Ok(Self {
            coefficient,
            scale: i32::try_from(scale).map_err(|_| ())?,
        })
    }

    fn canonical_string(&self) -> String {
        if self.coefficient == 0 {
            return "0".to_string();
        }
        let negative = self.coefficient < 0;
        let digits = self.coefficient.unsigned_abs().to_string();
        let mut output = if self.scale <= 0 {
            format!(
                "{}{}",
                digits,
                "0".repeat(self.scale.unsigned_abs() as usize)
            )
        } else if self.scale as usize >= digits.len() {
            format!(
                "0.{}{}",
                "0".repeat(self.scale as usize - digits.len()),
                digits
            )
        } else {
            let split = digits.len() - self.scale as usize;
            format!("{}.{}", &digits[..split], &digits[split..])
        };
        if negative {
            output.insert(0, '-');
        }
        output
    }

    fn cmp(&self, other: &Self) -> Ordering {
        if self.coefficient == 0 || other.coefficient == 0 {
            return self.coefficient.cmp(&other.coefficient);
        }
        let self_negative = self.coefficient < 0;
        let other_negative = other.coefficient < 0;
        if self_negative != other_negative {
            return self_negative.cmp(&other_negative).reverse();
        }

        let self_abs = self.coefficient.unsigned_abs();
        let other_abs = other.coefficient.unsigned_abs();
        let self_digits = self_abs.to_string();
        let other_digits = other_abs.to_string();
        let self_magnitude = self_digits.len() as i64 - i64::from(self.scale);
        let other_magnitude = other_digits.len() as i64 - i64::from(other.scale);
        let absolute_order = self_magnitude.cmp(&other_magnitude).then_with(|| {
            let length = self_digits.len().max(other_digits.len());
            (0..length)
                .map(|index| {
                    self_digits
                        .as_bytes()
                        .get(index)
                        .copied()
                        .unwrap_or(b'0')
                        .cmp(&other_digits.as_bytes().get(index).copied().unwrap_or(b'0'))
                })
                .find(|order| *order != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        });
        if self_negative {
            absolute_order.reverse()
        } else {
            absolute_order
        }
    }
}

/// Convert a decimal Unix timestamp expressed in seconds to exact nanoseconds.
/// Fractions finer than one nanosecond are rejected instead of being rounded by
/// an intermediate `f64`.
pub(crate) fn decimal_seconds_to_ns(input: &str) -> Result<i64, ()> {
    let decimal = Decimal::parse(input)?;
    let shift = 9i32 - decimal.scale;
    let nanoseconds = if shift >= 0 {
        let multiplier = 10_i128
            .checked_pow(u32::try_from(shift).map_err(|_| ())?)
            .ok_or(())?;
        decimal.coefficient.checked_mul(multiplier).ok_or(())?
    } else {
        let divisor = 10_i128.checked_pow(shift.unsigned_abs()).ok_or(())?;
        if decimal.coefficient % divisor != 0 {
            return Err(());
        }
        decimal.coefficient / divisor
    };
    i64::try_from(nanoseconds).map_err(|_| ())
}

#[derive(Debug, Clone)]
pub struct FieldFilter {
    pub name: String,
    pub op: FieldOp,
    pub value: FieldValue,
}

impl FieldFilter {
    fn matches(&self, fields: &BTreeMap<String, String>) -> bool {
        let actual = fields.get(&self.name).map(String::as_str).unwrap_or("");
        match (&self.op, &self.value) {
            (FieldOp::Eq, FieldValue::String(expected)) => actual == expected,
            (FieldOp::Neq, FieldValue::String(expected)) => actual != expected,
            (FieldOp::Regex, FieldValue::Regex(re)) => re.is_match(actual),
            (FieldOp::NotRegex, FieldValue::Regex(re)) => !re.is_match(actual),
            (op, FieldValue::Number(expected)) => Decimal::parse(actual)
                .ok()
                .is_some_and(|actual| compare_decimal(&actual, expected, *op)),
            (op, FieldValue::Duration(expected)) => parse_duration_ns(actual)
                .ok()
                .is_some_and(|actual| compare_ordered(actual, *expected, *op)),
            _ => false,
        }
    }
}

fn compare_decimal(actual: &Decimal, expected: &Decimal, op: FieldOp) -> bool {
    let ordering = actual.cmp(expected);
    match op {
        FieldOp::Eq => ordering == Ordering::Equal,
        FieldOp::Neq => ordering != Ordering::Equal,
        FieldOp::Lt => ordering == Ordering::Less,
        FieldOp::Lte => ordering != Ordering::Greater,
        FieldOp::Gt => ordering == Ordering::Greater,
        FieldOp::Gte => ordering != Ordering::Less,
        FieldOp::Regex | FieldOp::NotRegex => false,
    }
}

fn compare_ordered<T: PartialOrd + PartialEq>(actual: T, expected: T, op: FieldOp) -> bool {
    match op {
        FieldOp::Eq => actual == expected,
        FieldOp::Neq => actual != expected,
        FieldOp::Lt => actual < expected,
        FieldOp::Lte => actual <= expected,
        FieldOp::Gt => actual > expected,
        FieldOp::Gte => actual >= expected,
        FieldOp::Regex | FieldOp::NotRegex => false,
    }
}
