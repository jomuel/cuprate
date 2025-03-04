//! Database table abstraction; `trait Table`.

//---------------------------------------------------------------------------------------------------- Import

use crate::{key::Key, storable::Storable};

//---------------------------------------------------------------------------------------------------- Table
/// Database table metadata.
///
/// Purely compile time information for database tables.
///
/// ## Sealed
/// This trait is [`Sealed`](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed).
///
/// It is only implemented on the types inside [`tables`][crate::tables].
pub trait Table: crate::tables::private::Sealed + 'static {
    /// Name of the database table.
    const NAME: &'static str;

    /// Primary key type.
    type Key: Key + 'static;

    /// Value type.
    type Value: Storable + 'static;
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod test {
    // use super::*;
}
