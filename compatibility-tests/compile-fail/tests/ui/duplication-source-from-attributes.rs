 use snafu::Snafu;

 #[derive(Debug, Snafu)]
 enum EnumError {
     AVariant {
         // Should detect second attribute as duplicate
         #[snafu(source(from(EnumError, Box::new)))]
         #[snafu(source(from(EnumError, Box::new)))]
         my_source: Box<EnumError>,
     },
 }

fn main() {}
