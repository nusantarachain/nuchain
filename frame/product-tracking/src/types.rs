use codec::{Decode, Encode};
use core::fmt;
// use fixed::types::I16F16;
use frame_support::{sp_runtime::RuntimeDebug, sp_std::prelude::*, types::Property};
use pallet_product_registry::ProductId;

// use serde::{Serialize, Deserialize};

// Custom types
pub type Identifier = Vec<u8>;
// pub type Decimal = I16F16;
pub type Decimal = Vec<u8>;
pub type TrackingId = Identifier;
pub type TrackingEventIndex = u128;
pub type DeviceId = Identifier;

pub type TrackingStatus = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Track<AccountId, Moment> {
    pub id: TrackingId,
    pub owner: AccountId,
    pub status: TrackingStatus,
    pub products: Vec<ProductId>,
    pub registered: Moment,
    pub updated: Option<Moment>,
    /// parent tracking id yg merefer ke track sebelumnya apabila ada.
    pub parent_id: Option<TrackingId>,
    pub props: Option<Vec<Property>>,
}

impl<AccountId, Moment> Track<AccountId, Moment> {
    pub fn set_status(mut self, status: TrackingStatus) -> Self {
        self.status = status;
        self
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum TrackingEventType {
    TrackingRegistration,
    TrackingUpdateStatus,
    TrackingScan,
    TrackingDeliver,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct TrackingEvent<Moment> {
    pub event_type: TrackingEventType,
    pub tracking_id: TrackingId,
    pub location: Option<ReadPoint>,
    pub readings: Vec<Reading<Moment>>,
    pub status: TrackingStatus,
    pub timestamp: Moment,
}

// impl<Moment> fmt::Display for TrackingEvent<Moment>
// where
//     Moment: fmt::Debug,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // write!(f, "{:?}", self)

//         fn to_str(d: &Vec<u8>) -> str {
//             str::from_utf8(d).ok_or_else(|| "")
//         }

//         write!(f, r#"""
//         {{
//             "event_type": "{:?}",
//             "tracking_id": "{}",
//             "status": "{}",
//             "location": "{}",
//         }}
//         """#, self.event_type, 
//             to_str(&self.tracking_id),
//             self.status,
//             self.location)
//     }
// }


// impl<Moment> TrackingEvent<Moment> {
//     pub fn to_json(&self) -> Vec<u8> {
//         // let mut d = HashMap::new();
//         // d.insert("event_type", self.event_type.clone());

//         // serde_json_wasm::to_vec(&d).unwrap_or_else(|_| b"{}".to_vec())
        
//        vec![]
//     }
// }

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
