use hashbrown::HashMap;

use crate::quant::{ChannelId, Time};

#[derive(Debug)]
pub(super) struct Helper<'a> {
    all_channels: &'a [ChannelId],
    usage: ChannelUsage,
}

#[derive(Debug)]
enum ChannelUsage {
    Single(Time),
    Multiple(HashMap<ChannelId, Time>),
}

impl<'a> Helper<'a> {
    pub(super) fn new(all_channels: &'a [ChannelId]) -> Self {
        Self {
            all_channels,
            usage: if all_channels.is_empty() {
                ChannelUsage::Single(Time::ZERO)
            } else {
                ChannelUsage::Multiple(HashMap::with_capacity(all_channels.len()))
            },
        }
    }

    pub(super) fn get_usage(&self, channels: &[ChannelId]) -> Time {
        match &self.usage {
            ChannelUsage::Single(v) => *v,
            ChannelUsage::Multiple(d) => (if channels.is_empty() {
                d.values().max()
            } else {
                channels.iter().filter_map(|i| d.get(i)).max()
            })
            .copied()
            .unwrap_or_default(),
        }
    }

    pub(super) fn update_usage(&mut self, new_duration: Time, channels: &[ChannelId]) {
        let channels = if channels.is_empty() {
            self.all_channels
        } else {
            channels
        };
        match &mut self.usage {
            ChannelUsage::Single(v) => *v = new_duration,
            ChannelUsage::Multiple(d) => {
                for ch in channels {
                    d.insert(ch.clone(), new_duration);
                }
            }
        };
    }

    pub(super) fn into_max_usage(self) -> Time {
        match self.usage {
            ChannelUsage::Single(v) => v,
            ChannelUsage::Multiple(d) => d.into_values().max().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_no_channels() {
        let mut helper = Helper::new(&[]);
        assert_eq!(helper.get_usage(&[]), Time::ZERO);
        let time = Time::new(10.0).unwrap();
        helper.update_usage(time, &[]);
        assert_eq!(helper.get_usage(&[]), time);
        assert_eq!(helper.into_max_usage(), time);
    }

    #[test]
    fn test_helper_with_channels() {
        let channels = (0..5)
            .map(|i| ChannelId::new(i.to_string()))
            .collect::<Vec<_>>();
        let mut helper = Helper::new(&channels);
        assert_eq!(helper.get_usage(&[]), Time::ZERO);
        assert_eq!(helper.get_usage(&[channels[0].clone()]), Time::ZERO);

        let t1 = Time::new(10.0).unwrap();
        helper.update_usage(t1, &[]);
        assert_eq!(helper.get_usage(&[]), t1);
        assert_eq!(helper.get_usage(&[channels[0].clone()]), t1);

        let t2 = Time::new(20.0).unwrap();
        helper.update_usage(t2, &[channels[0].clone()]);
        assert_eq!(helper.get_usage(&[]), t2);
        assert_eq!(helper.get_usage(&[channels[0].clone()]), t2);
        assert_eq!(helper.get_usage(&[channels[1].clone()]), t1);
        assert_eq!(
            helper.get_usage(&[channels[0].clone(), channels[1].clone()]),
            t2
        );
        assert_eq!(helper.into_max_usage(), t2);
    }
}
