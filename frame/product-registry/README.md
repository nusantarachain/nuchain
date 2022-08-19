# Nuchain Product Registry

This pallet is intended to be used with supply chain functionality to register and managing product state between various stakeholders. This data is typically registered once by the product's manufacturer / supplier to be shared with other network participants.

It is inspired by existing projects & standards:
- [IBM Food Trust](https://github.com/IBM/IFT-Developer-Zone/wiki/APIs)
- [Hyperledger Grid](https://www.hyperledger.org/use/grid)
- [GS1 Standards](https://www.gs1.org/standards)


## Usage

To register a product, one must send a transaction with a `productRegistry.registerProduct` extrinsic with the following arguments:
- `id` as the Product ID, typically this would be a GS1 GTIN (Global Trade Item Number), or ASIN (Amazon Standard Identification Number), or similar, a numeric or alpha-numeric code with a well-defined data structure.
- `owner` as the Substrate Account representing the organization owning this product, as in the manufacturer or supplier providing this product within the value chain.
- `props` which is a series of properties (name & value) describing the product. Typically, there would at least be a textual description, and SKU. It could also contain instance / lot master data e.g. expiration, weight, harvest date.

## Dependencies

### Traits

This pallet depends on on the [FRAME EnsureOrigin System trait]
```
frame_support::traits::EnsureOrigin;
```

### Pallets

* Pallet timestamp ([pallet-timestamp](../timestamp))

## Testing

Run the tests with:

```
cargo test
```

### Genesis Configuration

This template pallet does not have any genesis configuration.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
