# derive-ctor

`derive-ctor` is a Rust procedural macro crate that allows you to easily generate constructor methods for your structs.
With the `#[derive(ctor)]` attribute, you can automatically create a constructor(s) for structs and enums. The crate also
provides various options to customize the generated constructor methods.

## Features
- Automatically generate a constructor method for structs and enums with `#[derive(ctor)]`.
- Customize the name and visibility of the auto-generated constructor using `#[ctor(visibility method_name)]`.
  - Supports const constructors by adding the "const" keyword.
  - Provide a list of names to generate multiple constructors.
- Customize field behavior in the constructor with the following properties (used in `#[ctor(PROPETY)])`:
  - **cloned** - Changes the parameter type to accept a reference type which is then cloned into the created struct.
  - **default** - Exclude the field from the generated method and use its default value.
  - **expr(EXPRESSION)** - Exclude the field from the generated method and use the defined expression as its default value.
    - **expr!(EXPRESSION)** to add the annotated field as a required parameter, allowing the expression to reference itself.
    - Use **expr(TYPE -> EXPRESSION)** to add a parameter with the specified type, which will be used to generate the final field value.
  - **into** - Change the parameter type for the generated method to `impl Into<Type>`.
  - **iter(FROM_TYPE)** - Change the parameter type for the generated method to `impl IntoIterator<Item=FROM_TYPE>`.
- No reliance on the standard library (no-std out of the box).

## Basic Usage

Add `derive-ctor` to your `Cargo.toml`:

```toml
[dependencies]
derive-ctor = "0.2.3"
```

Annotate your struct with `#[derive(ctor)]` to automatically generate a `new` constructor:

```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    field2: String
}

let my_struct = MyStruct::new(1, String::from("Foo"));
```

## Struct Configurations

### Visibility and Constructor Name

You can modify the name and visibility of the generated method, and define additional
constructors by using the `#[ctor]` attribute on the target struct after `ctor` is derived.

In the following example, three constructor methods are created: `new`, `with_defaults`, and `internal`.
These methods all inherit their respective visibilities defined within the `#[ctor]` attribute.

```rust
use derive_ctor::ctor;

#[derive(ctor)]
#[ctor(pub new, pub(crate) other, const internal)]
struct MyStruct {
    field1: i32,
    field2: String
}

let my_struct1 = MyStruct::new(100, "A".to_string());
let my_struct2 = MyStruct::other(200, "B".to_string());
let my_struct3 = MyStruct::internal(300, "C".to_string());
```

### Auto-implement "Default" Trait
The `Default` trait can be auto implemented by specifying a ctor with the name `default` in the ctor attribute. Note: all fields must have a generated value in order for the implementation to be valid.
Additionally, declaring `default(all)` will automatically mark all non-annotated fields with `#[ctor(default)]`

```rust
use derive_ctor::ctor;

#[derive(ctor)]
#[ctor(default)]
struct MyStruct {
    #[ctor(default)]
    field1: i32,
    #[ctor(expr(true))]
    field2: bool
}

let default: MyStruct = Default::default();


#[derive(ctor)]
#[ctor(default(all))]
struct OtherStruct {
    field1: i32,
    #[ctor(expr(true))]
    field2: bool
}

let default2: OtherStruct = Default::default();
```

## Enum Configurations

By default, a constructor will be generated for each variant. This constructor by default will match the name of its
respective variant and will be public. This default behaviour can be changed by annotating the enum with
`#[ctor(prefix = PREFIX, visibility = VISIBILITY)]`. Note that both parameters are optional within the attribute.
Specifying this attribute will change the **default** generated method for each variant, however, each variant
can additionally define its own configuration which overrides the one defined by the enum.

### Default variant constructor example

```rust
use derive_ctor::ctor;

#[derive(ctor)]
enum MyEnum {
    Variant1,
    Variant2(i32),
    Variant3 { value: bool }
}

let v1 = MyEnum::variant1();
let v2 = MyEnum::variant2(100);
let v3 = MyEnum::variant3(true);
```

### Configured variant constructor example
Variant constructor configuration is identical to struct constructor configuration. Refer to the below for
sample syntax or go-back to the struct constructor configuration for more information.

```rust
use derive_ctor::ctor;

#[derive(ctor)]
#[ctor(prefix = new, vis = pub(crate))]
enum MyEnum {
    #[ctor(const pub v1, other)]
    Variant1,
    Variant2,
    Variant3
}

const v1_1: MyEnum = MyEnum::v1();
let v1_2 = MyEnum::other();
let v2 = MyEnum::new_variant2();
let v3 = MyEnum::new_variant3();
```

If a variant is derived with `#[ctor(none)]` it will **not** have a constructor generated for it.

## Field Configurations

Fields can also be annotated with `#[ctor(PROPERTY)]` to change their behaviour in the generated methods.
**These configurations work for ALL enum-types and structs!**
The following are the available properties that can be used with the field-attributes

`#[ctor(cloned)]` - This property creates a parameter that accepts a type reference of the annotated field and
then clones it to generate the final value.
```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    #[ctor(cloned)]
    field2: String
}

let string = String::from("Foo");
let my_struct = MyStruct::new(100, &string);
```

`#[ctor(default)]` - This property excludes the annotated field from the constructor and uses its default value.
```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    #[ctor(default)]
    field2: String
}

let my_struct = MyStruct::new(100);
```

`#[ctor(into)]` - This property modifies the parameter type of the annotated field for the generated method
converting it from `Type` -> `impl Into<Type>`.
```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    #[ctor(into)] // the parameter type will now be impl Into<String> instead of String
    field2: String
}

let my_struct = MyStruct::new(100, "Foo");
```

`#[ctor(expr(EXPRESSION))]` - This property excludes the annotated field from the constructor and utilizes the defined expression
to generate its value.

**Alternatives:**

- `#[ctor(expr!(EXPRESSION))]` - Unlike the above attribute, this attribute will add the annotated field as a required parameter
for the given constructor, this allows for the provided EXPRESSION to reference the parameter and modify the passed value.
- `#[ctor(expr(TYPE -> EXPRESSION))]` - This attribute behaves similar to the variation above, however, the required parameter
type will be of the type provided in the attribute, thus allowing for a constructor to accept and map a parameter from one type
to the type used by the struct field.

```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    #[ctor(expr(String::from("Foo")))]
    field2: String,
    #[ctor(expr!(field3 + 100))]
    field3: u32,
    #[ctor(expr(i32 -> field4 < 0))]
    field4: bool
}

let my_struct = MyStruct::new(100, 5, -20); // generates MyStruct { field1: 100, field2: "foo", field3: 105, field4: true }
```

`#[ctor(iter(TYPE))]` - This property adds a parameter with the type: `impl IntoIterator<Item=TYPE>` and then generates
the annotated struct value by calling `.into_iter().collect()` on the parameter value.

```rust
use std::collections::HashSet;
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
  field1: i32,
  #[ctor(iter(usize))]
  field2: HashSet<usize>
}

let my_struct = MyStruct::new(0, vec![1, 1, 2, 3, 4]);
```

### Advanced Configuration

Field attributes can additionally be configured with a list of indices corresponding to the methods to use the generated
value for. This allows for the creation of multiple functions with different parameter requirements.

```rust
use derive_ctor::ctor;

#[derive(ctor)]
#[ctor(new, with_defaults)]
struct MyStruct {
  field1: i32,
  #[ctor(default = [1])]
  field2: String,
  #[ctor(default = 1)] // brackets can be removed if specifying only 1 index
  field3: bool,
  #[ctor(default = [0, 1])]
  field4: u64,
  #[ctor(default)] // this is the same as specifying all indices
  field5: u64
}

let my_struct1 = MyStruct::new(100, "Foo".to_string(), true);
let my_struct2 = MyStruct::with_defaults(100);
```