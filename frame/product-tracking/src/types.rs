use codec::{Decode, Encode};
use core::fmt;
use fixed::types::I16F16;
use frame_support::{sp_runtime::RuntimeDebug, sp_std::prelude::*};
use pallet_product_registry::ProductId;

// Custom types
pub type Identifier = Vec<u8>;
pub type Decimal = I16F16;
pub type TrackingId = Identifier;
pub type TrackingEventIndex = u128;
pub type DeviceId = Identifier;

// #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// pub enum TrackingStatus {
//     Pending,
//     InTransit,
//     Delivered,
// }

pub type TrackingStatus = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Track<AccountId, Moment> {
    pub id: TrackingId,
    pub owner: AccountId,
    pub status: TrackingStatus,
    pub products: Vec<ProductId>,
    pub registered: Moment,
    pub updated: Option<Moment>,
}

impl<AccountId, Moment> Track<AccountId, Moment> {
    // pub fn pickup(mut self) -> Self {
    //     self.status = TrackingStatus::InTransit;
    //     self
    // }

    // pub fn set_updated(mut self, updated_at: Moment) -> Self {
    //     self.status = TrackingStatus::updated;
    //     self.updated = Some(updated_at);
    //     self
    // }

    pub fn set_status(mut self, status: TrackingStatus) -> Self {
        self.status = status;
        self
    }
}

// #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// pub enum ShippingOperation {
//     Pickup,
//     Scan,
//     Deliver,
// }

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum TrackingEventType {
    TrackingRegistration,
    TrackingUpdateStatus,
    TrackingScan,
    TrackingDeliver,
}

// impl From<ShippingOperation> for TrackingEventType {
//     fn from(op: ShippingOperation) -> Self {
//         match op {
//             ShippingOperation::Pickup => TrackingEventType::TrackingPickup,
//             ShippingOperation::Scan => TrackingEventType::TrackingScan,
//             ShippingOperation::Deliver => TrackingEventType::TrackingDeliver,
//         }
//     }
// }

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct TrackingEvent<Moment> {
    pub event_type: TrackingEventType,
    pub tracking_id: TrackingId,
    pub location: Option<ReadPoint>,
    pub readings: Vec<Reading<Moment>>,
    pub status: TrackingStatus,
    pub timestamp: Moment,
}

impl<Moment> fmt::Display for TrackingEvent<Moment>
where
    Moment: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReadPoint {
    pub latitude: Decimal,
    pub longitude: Decimal,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReadingType {
    Humidity,
    Pressure,
    Shock,
    Tilt,
    Temperature,
    Vibration,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Reading<Moment> {
    pub device_id: DeviceId,
    pub reading_type: ReadingType,
    #[codec(compact)]
    pub timestamp: Moment,
    pub value: Decimal,
}
