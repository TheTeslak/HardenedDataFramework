#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RegionId(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BankId(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SectionId(pub &'static str);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PlacementSite {
    region: RegionId,
    bank: Option<BankId>,
    section: Option<SectionId>,
}

impl PlacementSite {
    pub const fn new(region: RegionId) -> Self {
        Self {
            region,
            bank: None,
            section: None,
        }
    }

    pub const fn with_details(
        region: RegionId,
        bank: Option<BankId>,
        section: Option<SectionId>,
    ) -> Self {
        Self {
            region,
            bank,
            section,
        }
    }

    pub const fn region(self) -> RegionId {
        self.region
    }

    pub const fn bank(self) -> Option<BankId> {
        self.bank
    }

    pub const fn section(self) -> Option<SectionId> {
        self.section
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplicaPlacement<const N: usize> {
    sites: [PlacementSite; N],
}

impl<const N: usize> ReplicaPlacement<N> {
    pub const fn new(regions: [RegionId; N]) -> Self {
        Self::from_regions(regions)
    }

    pub const fn from_regions(regions: [RegionId; N]) -> Self {
        let mut sites = [PlacementSite::new(RegionId(0)); N];
        let mut index = 0;
        while index < N {
            sites[index] = PlacementSite::new(regions[index]);
            index += 1;
        }
        Self { sites }
    }

    pub const fn with_sites(sites: [PlacementSite; N]) -> Self {
        Self { sites }
    }

    pub const fn sites(&self) -> &[PlacementSite; N] {
        &self.sites
    }

    pub fn regions(&self) -> [RegionId; N] {
        core::array::from_fn(|index| self.sites[index].region())
    }

    pub const fn site_of(&self, index: usize) -> Option<PlacementSite> {
        if index < N {
            Some(self.sites[index])
        } else {
            None
        }
    }

    pub const fn region_of(&self, index: usize) -> Option<RegionId> {
        if index < N {
            Some(self.sites[index].region())
        } else {
            None
        }
    }

    pub const fn bank_of(&self, index: usize) -> Option<BankId> {
        if index < N {
            self.sites[index].bank()
        } else {
            None
        }
    }

    pub const fn section_of(&self, index: usize) -> Option<SectionId> {
        if index < N {
            self.sites[index].section()
        } else {
            None
        }
    }
}
