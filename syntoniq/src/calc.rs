use anyhow::{anyhow, bail};
use clap::Subcommand;
use num_rational::Ratio;
use num_traits::ToPrimitive;
use std::collections::HashSet;
use syntoniq_common::parsing::score;
use syntoniq_common::pitch::{Factor, Pitch};

// *** NOTE ***
// Clap's help formatting is not very good, and this is a bit complex. The manual section on
// the Pitch Calculator describes this more comprehensively and should be kept up to date if this
// changes.
// *** NOTE ***

#[derive(Subcommand)]
#[command(verbatim_doc_comment)]
// TODO: this looks horrible with --help, and verbatim_doc_comment isn't helping.
pub enum Commands {
    /// Show information about scales that equally divide an interval. Columns:
    /// pitch: pitch explicitly shown with divisions as exponent denominator;
    /// simplified: pitch in canonical form;
    /// value: pitch as a floating point number;
    /// cents: pitch shown in cents;
    /// note: a note that would give this pitch in a generated scale with this many divisions;
    /// Δ scale degree: how far off the generated note's pure ratio is from the desired pitch in
    /// scale degrees;
    /// Δ cents: how far off the generated note's pure ratio is from the desired pitch in cents
    EqualScale {
        /// Interval to divide as a rational number in Syntoniq pitch notation;
        /// defaults to 2 (octave)
        #[arg(long)]
        interval: Option<String>,
        /// Number of divisions
        #[arg(long)]
        divisions: u32,
    },
    /// Find ratios or interval divisions within the given tolerance to the given pitch. This can
    /// be used to find ratios near pitches, scale degrees in an equal scale near ratios, etc.
    Near {
        #[arg(long)]
        pitch: String,
        /// Show near pitches as divisions of the given interval. If omitted and the pitch is a
        /// ratio, this defaults to 2. If omitted and the pitch is not a ratio, only ratios are
        /// included in the output.
        #[arg(long)]
        interval: Option<String>,
        /// When finding ratios, this is the maximum ratio denominator, reflecting how far up the
        /// harmonic series to search. When finding divisions of an interval, this is the maximum
        /// number of divisions to consider.
        #[arg(long)]
        max_denom: Option<u32>,
        /// How close to the pitch the ratio must be to be shown. The default is
        /// 1/75th of an octave, which is 16¢, about the amount by which a 12-tone
        /// minor third differs from the ratio 6/5.
        #[arg(long)]
        tolerance: Option<String>,
    },
    /// Compute the pitch resulting from multiplying all the arguments together. Arguments can be
    /// pitches in Syntoniq pitch notation or note names in the generated JI scale.
    Pitch { values: Vec<String> },
}

#[derive(Copy, Clone)]
enum Format {
    Left,
    Right,
    Char(&'static str),
}

/// Place the given strings in a tabular format. Each row must have the same number of columns,
/// which must be the length of `formats`. Text is aligned according to the corresponding format
/// as follows:
/// - Left - column is left-aligned
/// - Right - column is right-aligned
/// - Char(ch) - column is aligned on the occurrence of ch, if any; otherwise, align as if
///   the string ended with `ch`
///
/// Place `spaces` spaces between each column. If `headers` is non-empty, print centered headers
/// above each column, expanding column widths as needed to fit the headers.
fn format_tabular(
    headers: &[&str],
    formats: &[Format],
    spaces: usize,
    rows: &[Vec<String>],
) -> Vec<String> {
    // This function was written by Claude Sonnet 4.5 based on a small template, the doc comment,
    // and some example output.
    let num_cols = formats.len();
    for row in rows {
        if row.len() != num_cols {
            panic!("format_tabular called with inconsistent columns per row");
        }
    }
    if !headers.is_empty() && headers.len() != num_cols {
        panic!("format_tabular called with wrong number of headers");
    }

    if rows.is_empty() && headers.is_empty() {
        return vec![];
    }

    // Calculate column widths and alignment positions
    let mut col_widths = vec![0usize; num_cols];
    let mut char_positions: Vec<Vec<Option<usize>>> = vec![vec![]; num_cols];

    for (col, format) in formats.iter().enumerate() {
        for row in rows {
            let s = &row[col];
            col_widths[col] = col_widths[col].max(s.chars().count());

            if let Format::Char(ch) = format {
                char_positions[col].push(s.find(|c| ch.contains(c)));
            }
        }
    }

    // For Char alignment, compute max left-of-char width
    let mut max_left: Vec<usize> = vec![0; num_cols];
    let mut max_right: Vec<usize> = vec![0; num_cols];

    for (col, format) in formats.iter().enumerate() {
        if matches!(format, Format::Char(_)) {
            for (row_idx, row) in rows.iter().enumerate() {
                let s = &row[col];
                let n_chars = s.chars().count();
                let left = char_positions[col][row_idx].unwrap_or(n_chars);
                let right = n_chars - left;
                max_left[col] = max_left[col].max(left);
                max_right[col] = max_right[col].max(right);
            }
            col_widths[col] = max_left[col] + max_right[col];
        }
    }

    // Account for headers. If header is wider, we expand the column and track how much padding to
    // add on each side.
    let mut pad_left: Vec<usize> = vec![0; num_cols];
    let mut pad_right: Vec<usize> = vec![0; num_cols];

    if !headers.is_empty() {
        for (col, header) in headers.iter().enumerate() {
            let h_chars = header.chars().count();
            if h_chars > col_widths[col] {
                let extra = header.chars().count() - col_widths[col];
                // Pad on the right first, then on the left.
                pad_left[col] = extra / 2;
                pad_right[col] = extra - pad_left[col];
                col_widths[col] = h_chars;
            }
        }
    }

    let spacer = " ".repeat(spaces);

    let mut result = Vec::new();

    // Add header row if present
    if !headers.is_empty() {
        let header_row = headers
            .iter()
            .enumerate()
            .map(|(col, h)| {
                let width = col_widths[col];
                // Center the header within the column width
                let total_pad = width - h.chars().count();
                let left = total_pad / 2;
                let right = total_pad - left;
                format!("{}{}{}", " ".repeat(left), h, " ".repeat(right))
            })
            .collect::<Vec<_>>()
            .join(&spacer);
        result.push(header_row.trim_end().to_string());
    }

    // Add data rows
    for (row_idx, row) in rows.iter().enumerate() {
        let formatted_row = row
            .iter()
            .enumerate()
            .map(|(col, s)| {
                let inner_width = col_widths[col] - pad_left[col] - pad_right[col];
                let inner = match formats[col] {
                    Format::Left => format!("{:<inner_width$}", s),
                    Format::Right => format!("{:>inner_width$}", s),
                    Format::Char(_) => {
                        let n_chars = s.chars().count();
                        let left = char_positions[col][row_idx].unwrap_or(n_chars);
                        let char_pad_left = max_left[col] - left;
                        let char_pad_right = inner_width - char_pad_left - n_chars;
                        format!(
                            "{}{}{}",
                            " ".repeat(char_pad_left),
                            s,
                            " ".repeat(char_pad_right)
                        )
                    }
                };
                format!(
                    "{}{}{}",
                    " ".repeat(pad_left[col]),
                    inner,
                    " ".repeat(pad_right[col])
                )
            })
            .collect::<Vec<_>>()
            .join(&spacer);
        result.push(formatted_row.trim_end().to_string());
    }

    result
}

fn equal_scale(interval_ratio: Ratio<u32>, divisions: u32) -> Vec<String> {
    let octaves = interval_ratio.to_f64().unwrap().log(2.0);
    let cents_per_step: f64 = 1200.0 * octaves / divisions as f64;
    let mut rows = Vec::new();
    let mut names = score::generated_note_names(interval_ratio, divisions);
    names.push(("A'".to_string(), 0.0));
    for i in 0..=divisions {
        let p_str = format!("{interval_ratio}^{i}|{divisions}");
        let p = Pitch::must_parse(&p_str);
        let as_float = format!("{:.3}", p.as_float());
        let cents = format!("{:.3}¢", cents_per_step * i as f64);
        let (note, delta) = names[i as usize].clone();
        // Delta is number of scale degrees between the scale tone and the pure ratio.
        // A negative delta means you have to *add* the delta to the pure note to get the
        // scale note, so we invert the sign.
        fn sign(n: f64) -> char {
            if n <= 0.0 { '+' } else { '-' }
        }

        let note_clean = note.replace("#", "").replace("%", "") + "!";
        let offset_deg = format!("{note_clean} {} {:.3}°", sign(delta), delta.abs());
        let offset_cents = delta * cents_per_step;
        let offset_cents = format!(
            "{note_clean} {} {:6.3}¢",
            sign(offset_cents),
            offset_cents.abs()
        );
        rows.push(vec![
            p_str,
            p.to_string(),
            as_float,
            cents,
            note,
            offset_deg,
            offset_cents,
        ]);
    }
    format_tabular(
        &[
            "pitch",
            "simplified",
            "value",
            "cents",
            "note",
            "Δ scale degree",
            "Δ cents",
        ],
        &[
            Format::Char("|"),
            Format::Char("|"),
            Format::Char("."),
            Format::Char("."),
            Format::Right,
            Format::Char("+-"),
            Format::Char("+-"),
        ],
        3,
        &rows,
    )
}

fn find_nearest(
    orig_pitch: &str,
    pitch: Pitch,
    tolerance: Pitch,
    given_max_denom: Option<u32>,
    interval: Option<Ratio<u32>>,
) -> Vec<String> {
    let as_float = pitch.as_float();
    let max_denom;
    let target;
    let deviation_fn: fn(f64, f64) -> f64;

    if let Some(b) = interval {
        // We are looking for close pitches that divide the given interval `b`. To find the x such
        // that b^x is closest to a value, we find x that's closest to log_b(value). Since we are
        // working in a logarithmic scale, we use subtraction to determine the deviation. max_denom
        // is the largest number of divisions we are considering.
        max_denom = given_max_denom.unwrap_or(53);
        target = as_float.log(b.to_f64().unwrap());
        deviation_fn = |val, target| 1200.0 * (val - target);
    } else {
        // We want a ratio that is close to the pitch. We just have to find fractions close to the
        // floating point value of the pitch. max_denom is the largest ratio denominator, indicating
        // how far we go in the harmonic series.
        max_denom = given_max_denom.unwrap_or(32);
        target = as_float;
        deviation_fn = |val, target| (val / target).log2() * 1200.0
    }
    let max_deviation = tolerance.as_float().log2() * 1200.0;
    let mut seen = HashSet::new();
    struct Step1 {
        f: Ratio<u32>,
        deviation: f64,
    }
    let mut step1 = Vec::new();
    for den in 2..=max_denom {
        // Find the closest numerator for this denominator.
        let num = (target * den as f64).round();
        let val = num / den as f64;
        let deviation = deviation_fn(val, target);
        if deviation.abs() <= max_deviation {
            let f = Ratio::new(num as u32, den);
            if seen.insert(f) {
                step1.push(Step1 { f, deviation });
            }
        }
    }
    // Sort by absolute value of deviation. This is tricky because f64 doesn't implement Ord, but
    // we know we have no infinite/NAN values, so we can hack around it by unwrapping a partial_cmp.
    step1.sort_by(|a, b| a.deviation.abs().partial_cmp(&b.deviation.abs()).unwrap());
    struct Step2 {
        freq: f64,
        cents: f64,
        factor: Factor,
    }
    let best_matches: Vec<Step2> = step1
        .into_iter()
        .map(|v| match interval {
            Some(b) => {
                // The step1 ratio is the exponent of interval. Convert back to frequency scale.
                let freq = b.to_f64().unwrap().powf(v.f.to_f64().unwrap());
                let factor = Factor::new(
                    *b.numer(),
                    *b.denom(),
                    *v.f.numer() as i32,
                    *v.f.denom() as i32,
                )
                .unwrap();
                Step2 {
                    freq,
                    cents: v.deviation,
                    factor,
                }
            }
            None => Step2 {
                freq: v.f.to_f64().unwrap(),
                cents: v.deviation,
                factor: v.f.into(),
            },
        })
        .collect();
    let mut rows = Vec::new();
    for m in best_matches {
        let f_str = m.factor.to_string();
        let freq = format!("{:.3}", m.freq);
        let cents = format!("{:.3}¢", m.cents);
        rows.push(vec![f_str.clone(), freq.clone(), cents.clone()]);
        if interval.is_some()
            && let Some(pos) = f_str.find('^')
        {
            // Also express the exponent in non-simplified form for easier searching.
            let mut i = 2;
            let exp = m.factor.exponent();
            let num = *exp.numer() as u32;
            let den = *exp.denom() as u32;
            let prefix = &f_str[..=pos];
            while den * i < max_denom {
                let new_str = format!("{prefix}{}|{}", num * i, den * i);
                rows.push(vec![new_str, freq.clone(), format!("{cents} (= {f_str})")]);
                i += 1;
            }
        }
    }
    let mut output = vec![format!("== {orig_pitch} ≈ {as_float:.3} ==")];
    output.append(&mut format_tabular(
        &["pitch", "value", "Δ cents"],
        &[Format::Char("|/"), Format::Char("."), Format::Char(".")],
        3,
        &rows,
    ));
    output
}

fn calculate_pitch(values: Vec<String>) -> Vec<String> {
    fn fmt(rows: &[Vec<String>]) -> Vec<String> {
        format_tabular(&[], &[Format::Left, Format::Left], 2, rows)
    }

    let mut pitch = Pitch::unit();
    let mut rows = Vec::new();
    let mut errors = false;
    for v in values {
        match Pitch::parse(&v)
            .ok()
            .or_else(|| score::generated_note_pitch(&v))
        {
            Some(p) => {
                rows.push(vec![v, p.to_string()]);
                pitch *= &p;
            }
            None => {
                errors = true;
                rows.push(vec![v, "unable to parse as pitch or note".to_string()]);
            }
        }
    }
    if errors {
        return fmt(&rows);
    }
    rows.push(vec!["final pitch".to_string(), pitch.to_string()]);
    let as_float = pitch.as_float();
    rows.push(vec!["frequency".to_string(), format!("{as_float:.3}")]);
    if let Some(midi) = pitch.fractional_midi_note() {
        rows.push(vec!["MIDI note".to_string(), format!("{midi:.3}")]);
        let (note, bend) = pitch.midi().unwrap();
        rows.push(vec![
            "MPE (hex)".to_string(),
            format!("{note:02x}, {bend:04x}"),
        ]);
    } else {
        let octaves = as_float.log2();
        rows.push(vec!["octaves".to_string(), format!("{octaves:.3}")]);
        let cents = octaves * 1200.0;
        rows.push(vec!["cents".to_string(), format!("{cents:.3}¢")]);
    }
    fmt(&rows)
}

fn generate_output(command: Commands) -> anyhow::Result<Vec<String>> {
    let output = match command {
        Commands::EqualScale {
            interval,
            divisions,
        } => {
            let interval_ratio = match interval {
                None => Ratio::from_integer(2),
                Some(i) => Pitch::parse(&i)?
                    .as_rational()
                    .ok_or_else(|| anyhow!("interval must be rational"))?,
            };
            equal_scale(interval_ratio, divisions)
        }
        Commands::Near {
            pitch,
            interval,
            max_denom,
            tolerance,
        } => {
            let as_pitch = Pitch::parse(&pitch)?;
            if as_pitch.as_float() < 1.0 {
                bail!("pitch value must be > 1")
            }
            let tolerance = tolerance
                .map(|s| Pitch::parse(&s))
                .unwrap_or(Ok(Pitch::must_parse("^1|75")))?;
            // If no interval was specified and the pitch was rational, assume equal divisions of
            // the octave.
            let interval = match interval {
                None => as_pitch.as_rational().map(|_| Ratio::from_integer(2)),
                Some(s) => Some(s.parse::<Ratio<u32>>()?),
            };
            find_nearest(&pitch, as_pitch, tolerance, max_denom, interval)
        }
        Commands::Pitch { values } => calculate_pitch(values),
    };
    Ok(output)
}

pub fn run(command: Commands) -> anyhow::Result<()> {
    for line in generate_output(command)? {
        println!("{line}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_scale() {
        let out = generate_output(Commands::EqualScale {
            interval: Some("3".to_string()),
            divisions: 13,
        })
        .unwrap();
        assert_eq!(
            out,
            [
                " pitch    simplified   value     cents     note   Δ scale degree      Δ cents",
                " 3^0|13       1        1.000      0.000¢     A      A! + 0.000°     A! +  0.000¢",
                " 3^1|13     3^1|13     1.088    146.304¢     L      L! - 0.030°     L! -  4.333¢",
                " 3^2|13     3^2|13     1.184    292.608¢     F      F! - 0.157°     F! - 23.033¢",
                " 3^3|13     3^3|13     1.289    438.913¢    E#      E! + 0.360°     E! + 52.599¢",
                " 3^4|13     3^4|13     1.402    585.217¢    DT     DT! - 0.011°    DT! -  1.629¢",
                " 3^5|13     3^5|13     1.526    731.521¢    C#      C! + 0.202°     C! + 29.566¢",
                " 3^6|13     3^6|13     1.660    877.825¢    Bf     Bf! - 0.045°    Bf! -  6.533¢",
                " 3^7|13     3^7|13     1.807   1024.130¢    Bj     Bj! + 0.045°    Bj! +  6.533¢",
                " 3^8|13     3^8|13     1.966   1170.434¢    B%      B! - 0.202°     B! - 29.566¢",
                " 3^9|13     3^9|13     2.140   1316.738¢    BO     BO! - 0.018°    BO! -  2.705¢",
                "3^10|13    3^10|13     2.328   1463.042¢    BG     BG! - 0.026°    BG! -  3.829¢",
                "3^11|13    3^11|13     2.533   1609.347¢    BE     BE! + 0.157°    BE! + 23.033¢",
                "3^12|13    3^12|13     2.757   1755.651¢   BD#     BD! + 0.394°    BD! + 57.606¢",
                "3^13|13       3        3.000   1901.955¢    A'     A'! + 0.000°    A'! +  0.000¢",
            ]
        );

        let out = generate_output(Commands::EqualScale {
            interval: None,
            divisions: 19,
        })
        .unwrap();
        assert_eq!(
            out,
            [
                " pitch    simplified   value     cents     note   Δ scale degree      Δ cents",
                " 2^0|19       1        1.000      0.000¢     A      A! + 0.000°      A! +  0.000¢",
                " 2^1|19      ^1|19     1.037     63.158¢     Y      Y! - 0.119°      Y! -  7.515¢",
                " 2^2|19      ^2|19     1.076    126.316¢     N      N! - 0.031°      N! -  1.982¢",
                " 2^3|19      ^3|19     1.116    189.474¢     J      J! + 0.112°      J! +  7.070¢",
                " 2^4|19      ^4|19     1.157    252.632¢    G%      G! - 0.225°      G! - 14.239¢",
                " 2^5|19      ^5|19     1.200    315.789¢     F      F! + 0.002°      F! +  0.148¢",
                " 2^6|19      ^6|19     1.245    378.947¢     E      E! - 0.117°      E! -  7.366¢",
                " 2^7|19      ^7|19     1.291    442.105¢    FN     FN! - 0.029°     FN! -  1.834¢",
                " 2^8|19      ^8|19     1.339    505.263¢     D      D! + 0.114°      D! +  7.218¢",
                " 2^9|19      ^9|19     1.389    568.421¢    DY     DY! - 0.005°     DY! -  0.296¢",
                "2^10|19     ^10|19     1.440    631.579¢    Cy     Cy! + 0.005°     Cy! +  0.296¢",
                "2^11|19     ^11|19     1.494    694.737¢     C      C! - 0.114°      C! -  7.218¢",
                "2^12|19     ^12|19     1.549    757.895¢   Bfn    Bfn! + 0.029°    Bfn! +  1.834¢",
                "2^13|19     ^13|19     1.607    821.053¢    Be     Be! + 0.117°     Be! +  7.366¢",
                "2^14|19     ^14|19     1.667    884.211¢    Bf     Bf! - 0.002°     Bf! -  0.148¢",
                "2^15|19     ^15|19     1.728    947.368¢   Bg#     Bg! + 0.225°     Bg! + 14.239¢",
                "2^16|19     ^16|19     1.793   1010.526¢    Bj     Bj! - 0.112°     Bj! -  7.070¢",
                "2^17|19     ^17|19     1.859   1073.684¢    Bn     Bn! + 0.031°     Bn! +  1.982¢",
                "2^18|19     ^18|19     1.928   1136.842¢    By     By! + 0.119°     By! +  7.515¢",
                "2^19|19       2        2.000   1200.000¢    A'     A'! + 0.000°     A'! +  0.000¢",
            ]
        );
    }

    #[test]
    fn test_near() {
        let out = generate_output(Commands::Near {
            pitch: "5/4".to_string(),
            interval: None,
            max_denom: Some(19),
            tolerance: None,
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "== 5/4 ≈ 1.250 ==",
                "pitch   value        Δ cents",
                "^6|19   1.245    -7.366¢",
                "^5|16   1.242   -11.314¢",
                "^1|3    1.260    13.686¢",
                "^2|6    1.260    13.686¢ (= ^1|3)",
                "^3|9    1.260    13.686¢ (= ^1|3)",
                "^4|12   1.260    13.686¢ (= ^1|3)",
                "^5|15   1.260    13.686¢ (= ^1|3)",
                "^6|18   1.260    13.686¢ (= ^1|3)",
            ]
        );

        let out = generate_output(Commands::Near {
            pitch: "^6|17".to_string(),
            interval: None,
            max_denom: None,
            tolerance: None,
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "== ^6|17 ≈ 1.277 ==",
                "pitch   value   Δ cents",
                "23/18   1.278     0.835¢",
                "37/29   1.276    -1.763¢",
                "32/25   1.280     3.843¢",
                "41/32   1.281     5.533¢",
                "14/11   1.273    -6.021¢",
                "33/26   1.269   -10.784¢",
                " 9/7    1.286    11.555¢",
                "19/15   1.267   -14.285¢",
            ]
        );

        let out = generate_output(Commands::Near {
            pitch: "^7|12".to_string(),
            interval: Some("3".to_string()),
            max_denom: None,
            tolerance: None,
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "== ^7|12 ≈ 1.498 ==",
                " pitch    value         Δ cents",
                " 3^7|19   1.499     0.454¢",
                "3^14|38   1.499     0.454¢ (= 3^7|19)",
                "3^18|49   1.497    -0.835¢",
                "3^11|30   1.496    -1.651¢",
                "3^17|46   1.501     1.827¢",
                "3^15|41   1.495    -2.626¢",
                "3^10|27   1.502     2.794¢",
                "3^19|52   1.494    -3.189¢",
                "3^13|35   1.504     4.063¢",
                "3^16|43   1.505     4.861¢",
                " 3^4|11   1.491    -5.287¢",
                " 3^8|22   1.491    -5.287¢ (= 3^4|11)",
                "3^12|33   1.491    -5.287¢ (= 3^4|11)",
                "3^16|44   1.491    -5.287¢ (= 3^4|11)",
                "3^19|51   1.506     5.408¢",
                "3^17|47   1.488    -7.608¢",
                "3^13|36   1.487    -8.317¢",
                " 3^3|8    1.510     8.349¢",
                " 3^6|16   1.510     8.349¢ (= 3^3|8)",
                " 3^9|24   1.510     8.349¢ (= 3^3|8)",
                "3^12|32   1.510     8.349¢ (= 3^3|8)",
                "3^15|40   1.510     8.349¢ (= 3^3|8)",
                "3^18|48   1.510     8.349¢ (= 3^3|8)",
                " 3^9|25   1.485    -9.651¢",
                "3^18|50   1.485    -9.651¢ (= 3^9|25)",
                "3^14|39   1.483   -10.882¢",
                "3^20|53   1.514    11.179¢",
                "3^17|45   1.514    11.683¢",
                "3^14|37   1.515    12.403¢",
                " 3^5|14   1.480   -13.079¢",
                "3^10|28   1.480   -13.079¢ (= 3^5|14)",
                "3^15|42   1.480   -13.079¢ (= 3^5|14)",
                "3^11|29   1.517    13.522¢",
                " 3^8|21   1.520    15.492¢",
                "3^16|42   1.520    15.492¢ (= 3^8|21)",
                "3^11|31   1.477   -15.844¢",
            ]
        );

        let out = generate_output(Commands::Near {
            pitch: "^7|12".to_string(),
            interval: Some("3".to_string()),
            max_denom: Some(47),
            tolerance: Some("^1|150".to_string()),
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "== ^7|12 ≈ 1.498 ==",
                " pitch    value        Δ cents",
                " 3^7|19   1.499    0.454¢",
                "3^14|38   1.499    0.454¢ (= 3^7|19)",
                "3^11|30   1.496   -1.651¢",
                "3^17|46   1.501    1.827¢",
                "3^15|41   1.495   -2.626¢",
                "3^10|27   1.502    2.794¢",
                "3^13|35   1.504    4.063¢",
                "3^16|43   1.505    4.861¢",
                " 3^4|11   1.491   -5.287¢",
                " 3^8|22   1.491   -5.287¢ (= 3^4|11)",
                "3^12|33   1.491   -5.287¢ (= 3^4|11)",
                "3^16|44   1.491   -5.287¢ (= 3^4|11)",
                "3^17|47   1.488   -7.608¢",
            ]
        );
    }

    #[test]
    fn test_pitch() {
        let out = generate_output(Commands::Pitch {
            values: vec!["C!17".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "C!17         ^10|17",
                "final pitch  ^10|17",
                "frequency    1.503",
                "octaves      0.588",
                "cents        705.882¢",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["C!2/17".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "C!2/17       ^10|17",
                "final pitch  ^10|17",
                "frequency    1.503",
                "octaves      0.588",
                "cents        705.882¢",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["C!3/17".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "C!3/17       3^6|17",
                "final pitch  3^6|17",
                "frequency    1.474",
                "octaves      0.559",
                "cents        671.278¢",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["D!3/2/17".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "D!3/2/17     1/2*^5|17*3^12|17",
                "final pitch  1/2*^5|17*3^12|17",
                "frequency    1.331",
                "octaves      0.413",
                "cents        495.498¢",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["D!3/2/17/3".to_string()],
        })
        .unwrap();
        assert_eq!(out, ["D!3/2/17/3  unable to parse as pitch or note",]);

        let out = generate_output(Commands::Pitch {
            values: vec!["E!41".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "E!41         ^13|41",
                "final pitch  ^13|41",
                "frequency    1.246",
                "octaves      0.317",
                "cents        380.488¢",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["^4|17".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "^4|17        ^4|17",
                "final pitch  ^4|17",
                "frequency    1.177",
                "octaves      0.235",
                "cents        282.353¢",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["264".to_string(), "E!41".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "264          264",
                "E!41         ^13|41",
                "final pitch  264*^13|41",
                "frequency    328.891",
                "MIDI note    63.961",
                "MPE (hex)    40, 1ff9",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["220*^1|4".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "220*^1|4     220*^1|4",
                "final pitch  220*^1|4",
                "frequency    261.626",
                "MIDI note    60.000",
                "MPE (hex)    3c, 2000",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["220*^4|17".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "220*^4|17    220*^4|17",
                "final pitch  220*^4|17",
                "frequency    258.972",
                "MIDI note    59.824",
                "MPE (hex)    3c, 1fe2",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["440".to_string(), "E!41".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "440          440",
                "E!41         ^13|41",
                "final pitch  440*^13|41",
                "frequency    548.152",
                "MIDI note    72.805",
                "MPE (hex)    49, 1fdf",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["440*E!41".to_string()],
        })
        .unwrap();
        assert_eq!(out, ["440*E!41  unable to parse as pitch or note",]);

        let out = generate_output(Commands::Pitch {
            values: vec!["440*^-9|12".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "440*^-9|12   220*^1|4",
                "final pitch  220*^1|4",
                "frequency    261.626",
                "MIDI note    60.000",
                "MPE (hex)    3c, 2000",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["440*^-9|12".to_string(), "C!12".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "440*^-9|12   220*^1|4",
                "C!12         ^7|12",
                "final pitch  220*^5|6",
                "frequency    391.995",
                "MIDI note    67.000",
                "MPE (hex)    43, 2000",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["440*^-9|12".to_string(), "C!19".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "440*^-9|12   220*^1|4",
                "C!19         ^11|19",
                "final pitch  220*^63|76",
                "frequency    390.806",
                "MIDI note    66.947",
                "MPE (hex)    43, 1ff7",
            ]
        );

        let out = generate_output(Commands::Pitch {
            values: vec!["440*^-9|12".to_string(), "C".to_string()],
        })
        .unwrap();
        assert_eq!(
            out,
            [
                "440*^-9|12   220*^1|4",
                "C            3/2",
                "final pitch  330*^1|4",
                "frequency    392.438",
                "MIDI note    67.020",
                "MPE (hex)    43, 2003",
            ]
        );
    }
}
