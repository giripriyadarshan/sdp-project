use crate::models::{
    order_und_pagination::{OrderAndPagination, PageInfo},
    products::{
        paginate_products, Categories, Discounts, Products, ProductsPaginate, Reviews,
        ReviewsPaginate,
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

#[derive(Default)]
pub struct ProductsQuery;

#[Object]
impl ProductsQuery {
    async fn products_with_id(
        &self,
        ctx: &Context<'_>,
        category_id: Option<i32>,
        supplier_id: Option<i32>,
        base_product_id: Option<i32>,
        product_id: Option<i32>,
        paginator: OrderAndPagination,
    ) -> Result<ProductsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Products as ProductsEntity, products};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let products = ProductsEntity::find().filter(
            match (category_id, supplier_id, base_product_id, product_id) {
                (Some(category_id), None, None, None) => {
                    products::Column::CategoryId.eq(category_id)
                }
                (None, Some(supplier_id), None, None) => {
                    products::Column::SupplierId.eq(supplier_id)
                }
                (None, None, Some(base_product_id), None) => {
                    products::Column::BaseProductId.eq(base_product_id)
                }
                (None, None, None, Some(product_id)) => products::Column::ProductId.eq(product_id),
                _ => Err("Only one of category_id, supplier_id, base_product_id or product_id can be used")?,
            },
        );

        let products = paginate_products(paginator, products).await?;
        let products = products.paginate(db, page_size);
        let items = PageInfo {
            total_pages: products.num_pages().await?,
            total_items: products.num_items().await?,
        };

        let products = products.fetch_page(page).await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(ProductsPaginate {
            products,
            page_info: items,
        })
    }

    async fn products_with_name(
        &self,
        ctx: &Context<'_>,
        name: String,
        paginator: OrderAndPagination,
    ) -> Result<ProductsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Products as ProductsEntity, products};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let products = ProductsEntity::find().filter(products::Column::Name.contains(name));

        let products = paginate_products(paginator, products).await?;

        let products = products.paginate(db, page_size);
        let items = PageInfo {
            total_pages: products.num_pages().await?,
            total_items: products.num_items().await?,
        };

        let products = products.fetch_page(page).await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(ProductsPaginate {
            products,
            page_info: items,
        })
    }

    async fn categories(&self, ctx: &Context<'_>) -> Result<Vec<Categories>, async_graphql::Error> {
        use crate::entity::prelude::Categories as CategoriesEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let categories = CategoriesEntity::find().all(db).await?;

        let categories: Vec<Categories> = categories
            .into_iter()
            .map(|category| category.into())
            .collect();

        Ok(categories)
    }

    async fn reviews_for_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        paginator: OrderAndPagination,
    ) -> Result<ReviewsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Reviews as ReviewsEntity, reviews};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let reviews = ReviewsEntity::find().filter(reviews::Column::ProductId.eq(product_id));

        let reviews = reviews
            .order_by_asc(reviews::Column::ReviewDate)
            .paginate(db, page_size);

        let items = PageInfo {
            total_pages: reviews.num_pages().await?,
            total_items: reviews.num_items().await?,
        };

        let reviews = reviews.fetch_page(page).await?;

        let reviews: Vec<Reviews> = reviews.into_iter().map(|review| review.into()).collect();

        Ok(ReviewsPaginate {
            reviews,
            page_info: items,
        })
    }

    async fn discounts(&self, ctx: &Context<'_>) -> Result<Vec<Discounts>, async_graphql::Error> {
        use crate::entity::prelude::Discounts as DiscountsEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let discounts = DiscountsEntity::find().all(db).await?;

        let discounts: Vec<Discounts> = discounts
            .into_iter()
            .map(|discount| discount.into())
            .collect();

        Ok(discounts)
    }

    async fn discounts_on_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
    ) -> Result<Vec<Discounts>, async_graphql::Error> {
        use crate::entity::{discounts, prelude::Discounts as DiscountsEntity};
        let db = ctx.data::<DatabaseConnection>()?;

        let discounts = DiscountsEntity::find()
            .filter(discounts::Column::ProductId.eq(product_id))
            .all(db)
            .await?;

        let discounts: Vec<Discounts> = discounts
            .into_iter()
            .map(|discount| discount.into())
            .collect();

        Ok(discounts)
    }
}
