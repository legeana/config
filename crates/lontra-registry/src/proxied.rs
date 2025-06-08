#![allow(dead_code, unused_imports)]

pub(crate) trait Type {
    type Proxy;

    // Required.
    fn into_proxy(self) -> anyhow::Result<Self::Proxy>;
    fn to_proxy(&self) -> anyhow::Result<Self::Proxy>;
}

pub(crate) trait SizedType: Type + Sized {
    fn from_proxy(proxy: Self::Proxy) -> anyhow::Result<Self>;
}

// Internal types made public for macros only.
// Do not use directly.
pub(crate) mod internal {
    pub(crate) use sqlx::Database as SqlxDatabase;
    pub(crate) use sqlx::Decode as SqlxDecode;
    pub(crate) use sqlx::Encode as SqlxEncode;
    pub(crate) use sqlx::Type as SqlxType;
    pub(crate) use sqlx::encode::IsNull;
    pub(crate) use sqlx::error::BoxDynError;

    pub(crate) type BoxDynResult<T> = Result<T, BoxDynError>;
    pub(crate) type IsNullResult = BoxDynResult<IsNull>;
}

#[macro_export]
macro_rules! sqlx_type_impl {
    ($type:ty) => {
        impl<DB> $crate::proxied::internal::SqlxType<DB> for $type
        where
            DB: $crate::proxied::internal::SqlxDatabase,
            Self: $crate::proxied::Type,
            <Self as $crate::proxied::Type>::Proxy: $crate::proxied::internal::SqlxType<DB>,
        {
            fn type_info() -> <DB as $crate::proxied::internal::SqlxDatabase>::TypeInfo {
                use $crate::proxied::Type;
                use $crate::proxied::internal::SqlxType;

                <<Self as Type>::Proxy as SqlxType<DB>>::type_info()
            }
            fn compatible(ty: &<DB as $crate::proxied::internal::SqlxDatabase>::TypeInfo) -> bool {
                use $crate::proxied::Type;
                use $crate::proxied::internal::SqlxType;

                <<Self as Type>::Proxy as SqlxType<DB>>::compatible(ty)
            }
        }
    };
}

#[macro_export]
macro_rules! sqlx_decode_impl {
    ($type:ty) => {
        impl<'r, DB> $crate::proxied::internal::SqlxDecode<'r, DB> for $type
        where
            DB: $crate::proxied::internal::SqlxDatabase,
            Self: $crate::proxied::SizedType,
            <Self as $crate::proxied::Type>::Proxy: $crate::proxied::internal::SqlxDecode<'r, DB>,
        {
            fn decode(
                value: <DB as $crate::proxied::internal::SqlxDatabase>::ValueRef<'r>,
            ) -> $crate::proxied::internal::BoxDynResult<Self> {
                use $crate::proxied::SizedType;
                use $crate::proxied::Type;
                use $crate::proxied::internal::SqlxDecode;

                let proxy = <<Self as Type>::Proxy as SqlxDecode<'r, DB>>::decode(value)?;
                Ok(<Self as SizedType>::from_proxy(proxy)?)
            }
        }
    };
}

#[macro_export]
macro_rules! sqlx_encode_impl {
    ($type:ty) => {
        impl<'q, DB> $crate::proxied::internal::SqlxEncode<'q, DB> for $type
        where
            DB: $crate::proxied::internal::SqlxDatabase,
            Self: $crate::proxied::Type,
            <Self as $crate::proxied::Type>::Proxy: $crate::proxied::internal::SqlxEncode<'q, DB>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut <DB as $crate::proxied::internal::SqlxDatabase>::ArgumentBuffer<'q>,
            ) -> $crate::proxied::internal::IsNullResult {
                use $crate::proxied::Type as _;

                let proxy = self.to_proxy()?;
                proxy.encode_by_ref(buf)
            }
            fn encode(
                self,
                buf: &mut <DB as $crate::proxied::internal::SqlxDatabase>::ArgumentBuffer<'q>,
            ) -> $crate::proxied::internal::IsNullResult
            where
                Self: Sized,
            {
                use $crate::proxied::Type as _;

                let proxy = self.into_proxy()?;
                proxy.encode(buf)
            }
        }
    };
}

#[macro_export]
macro_rules! sqlx_unsized_impl {
    ($type:ty) => {
        $crate::sqlx_type_impl!($type);
        $crate::sqlx_encode_impl!($type);
    };
}

#[macro_export]
macro_rules! sqlx_impl {
    ($type:ty) => {
        $crate::sqlx_type_impl!($type);
        $crate::sqlx_decode_impl!($type);
        $crate::sqlx_encode_impl!($type);
    };
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use sqlx::Sqlite;

    use super::*;

    fn assert_type<T: sqlx::Type<Sqlite>>() {}
    fn assert_decode<'r, T: sqlx::Decode<'r, Sqlite>>() {}
    fn assert_encode<'q, T: sqlx::Encode<'q, Sqlite>>() {}

    // Tests for OwnedType.
    struct OwnedType;

    impl Type for OwnedType {
        type Proxy = Vec<u8>;

        fn into_proxy(self) -> Result<Self::Proxy> {
            Ok(Vec::new())
        }
        fn to_proxy(&self) -> Result<Self::Proxy> {
            Ok(Vec::new())
        }
    }

    impl SizedType for OwnedType {
        fn from_proxy(_proxy: Self::Proxy) -> Result<Self> {
            Ok(Self)
        }
    }

    crate::sqlx_impl!(OwnedType);

    #[test]
    fn test_owned() {
        assert_type::<OwnedType>();
        assert_decode::<OwnedType>();
        assert_encode::<OwnedType>();
    }

    // Tests for ReferenceType.
    struct ReferenceType<'a>(&'a str);

    impl<'a> Type for ReferenceType<'a> {
        type Proxy = &'a str;

        fn into_proxy(self) -> Result<Self::Proxy> {
            Ok(self.0)
        }
        fn to_proxy(&self) -> Result<Self::Proxy> {
            Ok(self.0)
        }
    }

    sqlx_unsized_impl!(ReferenceType<'_>);

    #[test]
    fn test_ref() {
        assert_type::<ReferenceType<'_>>();
        assert_encode::<ReferenceType<'_>>();
    }
}
