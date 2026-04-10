//! C-MAPSS dataset loader.
//!
//! Parses the whitespace-separated text format of C-MAPSS FD001--FD004.
//! Format: 26 columns per row:
//!   [unit, cycle, op1, op2, op3, s1, s2, ..., s21]

use std::io::{BufRead, BufReader, Read};
use std::collections::BTreeMap;

/// A single row from the C-MAPSS dataset.
#[derive(Debug, Clone)]
pub struct CmapssRow {
    /// Engine unit number (1-based).
    pub unit: u16,
    /// Cycle number (1-based).
    pub cycle: u32,
    /// Operational settings [altitude, Mach, TRA].
    pub op_settings: [f64; 3],
    /// 21 sensor measurements.
    pub sensors: [f64; 21],
}

/// Parsed C-MAPSS dataset (one of FD001--FD004).
#[derive(Debug)]
pub struct CmapssDataset {
    /// All rows sorted by (unit, cycle).
    pub rows: Vec<CmapssRow>,
    /// Dataset identifier (e.g., "FD001").
    pub name: String,
}

impl CmapssDataset {
    /// Parses a C-MAPSS text file from a reader.
    ///
    /// The format is whitespace-separated, 26 columns per line.
    /// No header row.
    pub fn parse<R: Read>(reader: R, name: &str) -> Result<Self, String> {
        let buf = BufReader::new(reader);
        let mut rows = Vec::new();

        for (line_idx, line_result) in buf.lines().enumerate() {
            let line = line_result.map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 26 {
                continue; // Skip malformed lines
            }

            let unit: u16 = parts[0].parse()
                .map_err(|_| format!("Line {}: invalid unit", line_idx + 1))?;
            let cycle: u32 = parts[1].parse()
                .map_err(|_| format!("Line {}: invalid cycle", line_idx + 1))?;

            let mut op_settings = [0.0f64; 3];
            for i in 0..3 {
                op_settings[i] = parts[2 + i].parse()
                    .map_err(|_| format!("Line {}: invalid op_setting {}", line_idx + 1, i + 1))?;
            }

            let mut sensors = [0.0f64; 21];
            for i in 0..21 {
                sensors[i] = parts[5 + i].parse()
                    .map_err(|_| format!("Line {}: invalid sensor {}", line_idx + 1, i + 1))?;
            }

            rows.push(CmapssRow { unit, cycle, op_settings, sensors });
        }

        Ok(Self { rows, name: name.to_string() })
    }

    /// Returns all unique engine unit numbers.
    pub fn units(&self) -> Vec<u16> {
        let mut seen = BTreeMap::new();
        for row in &self.rows {
            seen.entry(row.unit).or_insert(());
        }
        seen.keys().copied().collect()
    }

    /// Extracts a single sensor channel for a single engine unit.
    ///
    /// Returns values sorted by cycle, as a `Vec<f64>`.
    pub fn channel_for_unit(&self, unit: u16, sensor_index: usize) -> Vec<f64> {
        let mut values: Vec<(u32, f64)> = self.rows.iter()
            .filter(|r| r.unit == unit && sensor_index < 21)
            .map(|r| (r.cycle, r.sensors[sensor_index]))
            .collect();
        values.sort_by_key(|&(c, _)| c);
        values.into_iter().map(|(_, v)| v).collect()
    }

    /// Returns the maximum cycle for a given unit (= total lifetime).
    pub fn max_cycle(&self, unit: u16) -> u32 {
        self.rows.iter()
            .filter(|r| r.unit == unit)
            .map(|r| r.cycle)
            .max()
            .unwrap_or(0)
    }

    /// Number of engines in the dataset.
    pub fn num_units(&self) -> usize {
        self.units().len()
    }
}

/// Parses RUL (Remaining Useful Life) ground truth file.
///
/// One value per line, one per engine unit (in order).
pub fn parse_rul<R: Read>(reader: R) -> Result<Vec<u32>, String> {
    let buf = BufReader::new(reader);
    let mut ruls = Vec::new();
    for (i, line_result) in buf.lines().enumerate() {
        let line = line_result.map_err(|e| format!("RUL line {}: {}", i + 1, e))?;
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let val: u32 = trimmed.parse()
            .map_err(|_| format!("RUL line {}: invalid value", i + 1))?;
        ruls.push(val);
    }
    Ok(ruls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple() {
        let data = "1 1 -0.0007 -0.0004 100.0 518.67 641.82 1589.70 1400.60 14.62 21.61 554.36 2388.06 9046.19 1.30 47.47 521.66 2388.02 8138.62 8.4195 0.03 392 2388 100.0 39.06 23.42\n\
                     1 2 -0.0007 -0.0004 100.0 518.67 642.15 1591.82 1403.14 14.62 21.61 553.75 2388.04 9044.07 1.30 47.49 522.28 2388.07 8131.49 8.4318 0.03 392 2388 100.0 39.00 23.36\n";
        let cursor = Cursor::new(data);
        let ds = CmapssDataset::parse(cursor, "test").unwrap();
        assert_eq!(ds.rows.len(), 2);
        assert_eq!(ds.rows[0].unit, 1);
        assert_eq!(ds.rows[0].cycle, 1);
    }
}
