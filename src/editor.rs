use crate::analyzer::Segment;

pub fn calculate_keep_segments(silence_segments: &[Segment], total_duration: f32) -> Vec<Segment> {
    let mut keep = Vec::new();
    let mut current_pos = 0.0;

    for silence in silence_segments {
        if silence.start > current_pos {
            keep.push(Segment {
                start: current_pos,
                end: silence.start,
            });
        }
        current_pos = silence.end;
    }

    if current_pos < total_duration {
        keep.push(Segment {
            start: current_pos,
            end: total_duration,
        });
    }

    keep
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_keep_segments() {
        let silences = vec![
            Segment { start: 1.0, end: 2.0 },
            Segment { start: 4.0, end: 5.0 },
        ];
        let duration = 10.0;
        let keeps = calculate_keep_segments(&silences, duration);

        assert_eq!(keeps.len(), 3);
        assert_eq!(keeps[0], Segment { start: 0.0, end: 1.0 });
        assert_eq!(keeps[1], Segment { start: 2.0, end: 4.0 });
        assert_eq!(keeps[2], Segment { start: 5.0, end: 10.0 });
    }
}
