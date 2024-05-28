# derive-ctor

`derive-ctor` is a Rust procedural macro crate that allows you to easily generate constructor methods for your structs. With the `#[derive(ctor)]` attribute, you can automatically create a constructor for all fields in the struct. The crate also provides various options to customize the generated constructor methods.

## Features

- Automatically generate a constructor method for all fields in a struct with `#[derive(ctor)]`.
- Customize the name and visibility of the auto-generated constructor using `#[ctor(visibility method_name)]`.
- Provide a list of names to generate multiple constructors.
- Customize field behavior in the constructor with the following attributes:
  - `#[ctor(default)]` - Exclude the field from the generated method and use its default value.
  - `#[ctor(expr(EXPRESSION))]` - Exclude the field from the generated method and use the defined expression as its default value.
  - `#[ctor(impl)]` - Change the parameter type for the generated method to `impl Into<Type>`.

## Basic Usage

Add `derive-ctor` to your `Cargo.toml`:

```toml
[dependencies]
derive-ctor = "0.1"
```

Import the crate in your Rust code:
```rust
use derive_ctor::ctor;
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

## Configurations

You can modify the name and visibility of the generated method, and define additional
constructors by using the `#[ctor]` attribute on the target struct after `ctor` is derived.

In the following example, three constructor methods are created: `new`, `with_defaults`, and `internal`.
These methods all inherit their respective visibilities defined within the `#[ctor]` attribute.

```rust
use derive_ctor::ctor;

#[derive(ctor)]
#[ctor(pub new, pub(crate) with_defaults, internal)]
struct MyStruct {
    field1: i32,
    field2: String
}
```

### Field Configurations

Fields can also be annotated with `#[ctor(PROPERTY)]` to change their behaviour in the generated methods.
The following are the available properties that can be used with the field-attributes

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

`#[ctor(impl)]` - This property modifies the parameter type of the annotated field for the generated method
converting it from `Type` -> `impl Into<Type>`.
```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    #[ctor(impl)] // the parameter type will now be impl Into<String> instead of String
    field2: String
}

let my_struct = MyStruct::new(100, "Foo");
```

`#[ctor(expr(VALUE))]` - This property excludes the annotated field from the constructor and utilizes the defined expression
to generate its value.
```rust
use derive_ctor::ctor;

#[derive(ctor)]
struct MyStruct {
    field1: i32,
    #[ctor(expr(String::from("Foo")))]
    field2: String
}

let my_struct = MyStruct::new(100); // generates MyStruct { field1: 100, field2: "foo" }
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