// #[derive(Debug, Clone)]
// pub struct Sequence {
//     pub title: String,
//     pub elements: Vec<SequenceElement>,
//     pub time_signature: TimeSignature,
// }

// impl Sequence {
//     pub fn new<T>(title: &str, time_signature: T) -> Self
//     where
//         T: Into<TimeSignature>,
//     {
//         let time_signature: TimeSignature = time_signature.into();
//         Sequence {
//             title: title.to_string(),
//             elements: Vec::new(),
//             time_signature,
//         }
//     }

//     pub fn concatenate(mut self, other: &Sequence) -> Self {
//         self.elements.extend(other.elements.clone());
//         self
//     }
// }
