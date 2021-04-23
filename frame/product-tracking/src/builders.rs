use crate::types::*;
use frame_support::sp_std::prelude::*;
use pallet_product_registry::ProductId;

// --- TrackingBuilder ---

#[derive(Default)]
pub struct TrackingBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
    id: TrackingId,
    owner: AccountId,
    products: Vec<ProductId>,
    registered: Moment,
}

impl<AccountId, Moment> TrackingBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
    pub fn identified_by(mut self, id: TrackingId) -> Self {
        self.id = id;
        self
    }

    pub fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_products(mut self, products: Vec<ProductId>) -> Self {
        self.products = products;
        self
    }

    pub fn registered_at(mut self, registered: Moment) -> Self {
        self.registered = registered;
        self
    }

    pub fn build(self) -> Track<AccountId, Moment> {
        Track::<AccountId, Moment> {
            id: self.id,
            owner: self.owner,
            products: self.products,
            registered: self.registered,
            status: b"".to_vec(),
            updated: None,
        }
    }
}

// --- TrackingEventBuilder ---

pub struct TrackingEventBuilder<Moment>
where
    Moment: Default,
{
    tracking_id: TrackingId,
    event_type: TrackingEventType,
    location: Option<ReadPoint>,
    readings: Vec<Reading<Moment>>,
    status: TrackingStatus,
    timestamp: Moment,
}

impl<Moment> Default for TrackingEventBuilder<Moment>
where
    Moment: Default,
{
    fn default() -> Self {
        TrackingEventBuilder {
            tracking_id: TrackingId::default(),
            event_type: TrackingEventType::TrackingUpdateStatus,
            location: Option::<ReadPoint>::default(),
            readings: Vec::<Reading<Moment>>::default(),
            status: b"registered".to_vec(),
            timestamp: Moment::default(),
        }
    }
}

impl<Moment> TrackingEventBuilder<Moment>
where
    Moment: Default,
{
    pub fn of_type(mut self, event_type: TrackingEventType) -> Self {
        self.event_type = event_type;
        self
    }

    pub fn for_tracking(mut self, id: TrackingId) -> Self {
        self.tracking_id = id;
        self
    }

    pub fn at_location(mut self, location: Option<ReadPoint>) -> Self {
        self.location = location;
        self
    }

    pub fn with_readings(mut self, readings: Vec<Reading<Moment>>) -> Self {
        self.readings = readings;
        self
    }

    pub fn at_time(mut self, timestamp: Moment) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_status(mut self, status: TrackingStatus) -> Self {
        self.status = status;
        self
    }

    pub fn build(self) -> TrackingEvent<Moment> {
        TrackingEvent::<Moment> {
            event_type: self.event_type,
            tracking_id: self.tracking_id,
            location: self.location,
            readings: self.readings,
            status: self.status,
            timestamp: self.timestamp,
        }
    }
}
