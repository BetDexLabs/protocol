<!-- Generated by documentation.js. Update this documentation by updating the source code. -->

### Table of Contents

*   [Products][1]
    *   [Parameters][2]
    *   [Examples][3]

## Products

**Extends AccountQuery**

Base product query builder allowing to filter by set fields. Returns publicKeys or accounts mapped to those publicKeys; filtered to remove any accounts closed during the query process.

### Parameters

*   `program`  {program} protocol\_product program initialized by the consuming client

### Examples

```javascript
const authority = new PublicKey('7o1PXyYZtBBDFZf9cEhHopn2C9R4G6GaPwFAxaNWM33D')
const payer = new PublicKey('5BZWY6XWPxuWFxs2jagkmUkCoBWmJ6c4YEArr83hYBWk')
const products = await Products.productQuery(program)
      .filterByPayer(marketPk)
      .filterByAuthority(purchaserPk)
      .fetch();

// Returns all open product accounts for the specified payer and authority.
```

[1]: #products

[2]: #parameters

[3]: #examples
