// use hashbrown::HashMap;

// use crate::{
//     pulse::{Envelope, PulseList, PulseListBuilder},
//     quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time},
//     schedule::{self, ElementVariant},
//     shape::Shape,
// };

// #[derive(Debug, Clone)]
// pub(crate) struct Executor {
//     channels: HashMap<ChannelId, Channel>,
//     shapes: HashMap<ShapeId, Shape>,
//     amp_tolerance: Amplitude,
//     time_tolerance: Time,
// }

// impl Executor {
//     pub(crate) fn new(amp_tolerance: Amplitude, time_tolerance: Time) -> Self {
//         Self {
//             channels: HashMap::new(),
//             shapes: HashMap::new(),
//             amp_tolerance,
//             time_tolerance,
//         }
//     }

//     pub(crate) fn add_channel(&mut self, name: ChannelId, base_freq: Frequency) {
//         self.channels.insert(
//             name,
//             Channel::new(base_freq, self.amp_tolerance, self.time_tolerance),
//         );
//     }

//     pub(crate) fn add_shape(&mut self, name: ShapeId, shape: Shape) {
//         self.shapes.insert(name, shape);
//     }

//     pub(crate) fn execute(&mut self, element: &ArrangedElement) {
//         self.execute_dispatch(element, Time::ZERO);
//     }

//     pub(crate) fn into_result(self) -> HashMap<ChannelId, PulseList> {
//         self.channels
//             .into_iter()
//             .map(|(n, b)| (n, b.pulses.build()))
//             .collect()
//     }

//     fn execute_dispatch(&mut self, element: &ArrangedElement, time: Time) {
//         if element.element().common.phantom() {
//             return;
//         }
//         let time = time + element.inner_time();
//         let duration = element.inner_duration();
//         match &element.element().variant {
//             ElementVariant::Play(e) => self.execute_play(e, time, duration),
//             ElementVariant::ShiftPhase(e) => self.execute_shift_phase(e),
//             ElementVariant::SetPhase(e) => self.execute_set_phase(e, time),
//             ElementVariant::ShiftFreq(e) => self.execute_shift_freq(e, time),
//             ElementVariant::SetFreq(e) => self.execute_set_freq(e, time),
//             ElementVariant::SwapPhase(e) => self.execute_swap_phase(e, time),
//             ElementVariant::Barrier(_) => (),
//             ElementVariant::Repeat(e) => {
//                 let child = &element.try_get_children().expect("Invalid arrange data")[0];
//                 self.execute_repeat(e, child, time, duration);
//             }
//             ElementVariant::Stack(_) | ElementVariant::Absolute(_) | ElementVariant::Grid(_) => {
//                 let children = element.try_get_children().expect("Invalid arrange data");
//                 self.execute_container(children, time);
//             }
//         }
//     }

//     fn execute_play(&mut self, element: &schedule::Play, time: Time, duration: Time) {
//         let shape = element.shape_id().map(|id| self.shapes[id].clone());
//         let width = element.width();
//         let plateau = if element.flexible() {
//             duration - width
//         } else {
//             element.plateau()
//         };
//         let amplitude = element.amplitude();
//         let drag_coef = element.drag_coef();
//         let freq = element.frequency();
//         let phase = element.phase();
//         let channel = self.channels.get_mut(element.channel_id()).unwrap();
//         channel.add_pulse(
//             shape, time, width, plateau, amplitude, drag_coef, freq, phase,
//         );
//     }

//     fn execute_shift_phase(&mut self, element: &schedule::ShiftPhase) {
//         let delta_phase = element.phase();
//         let channel = self.channels.get_mut(element.channel_id()).unwrap();
//         channel.shift_phase(delta_phase);
//     }

//     fn execute_set_phase(&mut self, element: &schedule::SetPhase, time: Time) {
//         let phase = element.phase();
//         let channel = self.channels.get_mut(element.channel_id()).unwrap();
//         channel.set_phase(phase, time);
//     }

//     fn execute_shift_freq(&mut self, element: &schedule::ShiftFreq, time: Time) {
//         let delta_freq = element.frequency();
//         let channel = self.channels.get_mut(element.channel_id()).unwrap();
//         channel.shift_freq(delta_freq, time);
//     }

//     fn execute_set_freq(&mut self, element: &schedule::SetFreq, time: Time) {
//         let freq = element.frequency();
//         let channel = self.channels.get_mut(element.channel_id()).unwrap();
//         channel.set_freq(freq, time);
//     }

//     fn execute_swap_phase(&mut self, element: &schedule::SwapPhase, time: Time) {
//         let ch1 = element.channel_id1();
//         let ch2 = element.channel_id2();
//         if ch1 == ch2 {
//             return;
//         }
//         let [channel, other] = self.channels.get_many_mut([ch1, ch2]).unwrap();
//         channel.swap_phase(other, time);
//     }

//     fn execute_repeat(
//         &mut self,
//         element: &schedule::Repeat,
//         child: &ArrangedElement,
//         time: Time,
//         duration: Time,
//     ) {
//         let count = element.count();
//         if count == 0 {
//             return;
//         }
//         let spacing = element.spacing();
//         let time_step = (duration + spacing) / count as f64;
//         for i in 0..count {
//             let child_time = time + i as f64 * time_step;
//             self.execute_dispatch(child, child_time);
//         }
//     }

//     fn execute_container(&mut self, children: &[ArrangedElement], time: Time) {
//         for child in children {
//             self.execute_dispatch(child, time);
//         }
//     }
// }

// #[derive(Debug, Clone)]
// struct Channel {
//     base_freq: Frequency,
//     delta_freq: Frequency,
//     phase: Phase,
//     pulses: PulseListBuilder,
// }

// impl Channel {
//     fn new(base_freq: Frequency, amp_tolerance: Amplitude, time_tolerance: Time) -> Self {
//         Self {
//             base_freq,
//             delta_freq: Frequency::ZERO,
//             phase: Phase::ZERO,
//             pulses: PulseListBuilder::new(amp_tolerance, time_tolerance),
//         }
//     }

//     fn shift_freq(&mut self, delta_freq: Frequency, time: Time) {
//         let delta_phase = -delta_freq * time;
//         self.delta_freq += delta_freq;
//         self.phase += delta_phase;
//     }

//     fn set_freq(&mut self, freq: Frequency, time: Time) {
//         let delta_freq = freq - self.delta_freq;
//         let delta_phase = -delta_freq * time;
//         self.delta_freq = freq;
//         self.phase += delta_phase;
//     }

//     fn shift_phase(&mut self, delta_phase: Phase) {
//         self.phase += delta_phase;
//     }

//     fn set_phase(&mut self, phase: Phase, time: Time) {
//         self.phase = phase - self.delta_freq * time;
//     }

//     fn total_freq(&self) -> Frequency {
//         self.base_freq + self.delta_freq
//     }

//     fn swap_phase(&mut self, other: &mut Self, time: Time) {
//         let delta_freq = self.total_freq() - other.total_freq();
//         let phase1 = self.phase;
//         let phase2 = other.phase;
//         self.phase = phase2 - delta_freq * time;
//         other.phase = phase1 + delta_freq * time;
//     }

//     fn add_pulse(
//         &mut self,
//         shape: Option<Shape>,
//         time: Time,
//         width: Time,
//         plateau: Time,
//         amplitude: Amplitude,
//         drag_coef: f64,
//         freq: Frequency,
//         phase: Phase,
//     ) {
//         let envelope = Envelope::new(shape, width, plateau);
//         let global_freq = self.total_freq();
//         let local_freq = freq;
//         self.pulses.push(
//             envelope,
//             global_freq,
//             local_freq,
//             time,
//             amplitude,
//             drag_coef,
//             phase,
//         )
//     }
// }
