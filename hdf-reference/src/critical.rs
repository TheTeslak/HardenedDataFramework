use hdf_sync::CriticalSection;

#[derive(Clone, Copy, Debug, Default)]
pub struct DemoCriticalSection;

impl CriticalSection for DemoCriticalSection {
    fn enter<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }
}
