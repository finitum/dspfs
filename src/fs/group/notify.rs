// use anyhow::Result;
//
// use async_trait::async_trait;

// #[async_trait]
// pub trait Notify {
//     async fn file_added(&mut self, file: &shared::File) -> Result<()>;
// }

// #[cfg(test)]
// pub mod mock {
//     use crate::dspfs::notify::Notify;
//     use anyhow::Result;
//     use async_trait::async_trait;
//
//     pub enum Notification {
//         FileAdded(File),
//     }
//
//     pub struct NotifyMock {
//         notifications: Vec<Notification>,
//     }
//
//     impl NotifyMock {
//         pub fn new() -> Self {
//             Self {
//                 notifications: vec![],
//             }
//         }
//
//         pub fn get(&self, index: usize) -> Option<&Notification> {
//             self.notifications.get(index)
//         }
//     }
//
//     #[async_trait]
//     impl Notify for NotifyMock {
//         async fn file_added(&mut self, file: &File) -> Result<()> {
//             self.notifications
//                 .push(Notification::FileAdded(file.to_owned()));
//             Ok(())
//         }
//     }
// }
