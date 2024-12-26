create type payment_method_type as enum ('netbanking', 'card', 'iban', 'upi');

create type user_role as enum ('customer', 'supplier');

create table categories
(
    category_id        serial
        primary key,
    name               varchar(50) not null,
    parent_category_id integer
        constraint fk_parent_category
            references categories
            on delete set null
);

create table card_types
(
    card_type_id serial
        primary key,
    name         varchar(20) not null
        unique
);

create table address_types
(
    address_type_id serial
        primary key,
    name            varchar(20) not null
        unique
);

create table users
(
    user_id        serial
        primary key,
    email          varchar(100) not null
        unique,
    password       varchar(255) not null,
    role           user_role    not null
        constraint users_role_check
            check ((role)::text = ANY
                   (ARRAY [('customer'::character varying)::text, ('supplier'::character varying)::text])),
    created_at     timestamp with time zone default CURRENT_TIMESTAMP,
    email_verified boolean                  default false
);

create table customers
(
    customer_id       serial
        primary key,
    first_name        varchar(50) not null,
    last_name         varchar(50) not null,
    registration_date timestamp with time zone default CURRENT_TIMESTAMP,
    user_id           integer     not null
        unique
        constraint fk_user_customer
            references users
            on delete cascade
);

create table addresses
(
    address_id      serial
        primary key,
    customer_id     integer      not null
        constraint fk_customer
            references customers
            on delete cascade,
    street_address  varchar(100) not null,
    city            varchar(50)  not null,
    state           varchar(50),
    postal_code     char(10)     not null,
    country         char(3)      not null,
    is_default      boolean default false,
    address_type_id integer
        constraint fk_address_type
            references address_types,
    constraint unique_default_address
        unique (customer_id, is_default)
);

create index idx_addresses_customer_default
    on addresses (customer_id, is_default);

create table payment_methods
(
    payment_method_id    serial
        primary key,
    customer_id          integer             not null
        constraint fk_customer
            references customers
            on delete cascade,
    payment_type         payment_method_type not null,
    is_default           boolean default false,
    bank_name            varchar(100),
    account_holder_name  varchar(100),
    card_number          char(16),
    card_expiration_date date,
    iban                 char(34),
    upi_id               varchar(50),
    bank_account_number  varchar(20),
    ifsc_code            varchar(11),
    card_type_id         integer
        constraint fk_card_type
            references card_types
);

create index idx_payment_methods_customer_default
    on payment_methods (customer_id, is_default);

create unique index idx_unique_default_payment_method
    on payment_methods (customer_id)
    where (is_default = true);

create table suppliers
(
    supplier_id   serial
        primary key,
    name          varchar(100) not null,
    contact_phone text,
    user_id       integer      not null
        unique
        constraint fk_user_supplier
            references users
            on delete cascade
);

create table products
(
    product_id      serial
        primary key,
    name            varchar(100)      not null,
    description     text,
    base_price      numeric(10, 2)    not null,
    category_id     integer
        constraint fk_category
            references categories
            on delete set null,
    supplier_id     integer
        constraint fk_supplier
            references suppliers
            on delete set null,
    stock_quantity  integer default 0 not null,
    base_product_id integer
        constraint fk_base_product
            references products
            on delete set null,
    media_paths     text[],
    created_at      timestamp with time zone
);

create index idx_product_category
    on products (category_id);

create index idx_product_supplier
    on products (supplier_id);

create index idx_product_name
    on products (name);

create table shopping_carts
(
    cart_id     serial
        primary key,
    customer_id integer not null
        constraint fk_customer
            references customers
            on delete cascade,
    created_at  timestamp with time zone default CURRENT_TIMESTAMP
);

create table cart_items
(
    cart_item_id serial
        primary key,
    cart_id      integer not null
        constraint fk_cart
            references shopping_carts
            on delete cascade,
    product_id   integer not null
        constraint fk_product
            references products
            on delete cascade,
    quantity     integer not null
);

create index idx_cart_items_cart
    on cart_items (cart_id);

create index idx_cart_items_product
    on cart_items (product_id);

create table reviews
(
    review_id   serial
        primary key,
    customer_id integer not null
        constraint fk_customer_review
            references customers
            on delete cascade,
    product_id  integer not null
        constraint fk_product_review
            references products
            on delete cascade,
    rating      integer
        constraint reviews_rating_check
            check ((rating >= 1) AND (rating <= 5)),
    review_text text,
    review_date timestamp with time zone default CURRENT_TIMESTAMP,
    media_paths text[]
);

create index idx_reviews_product
    on reviews (product_id);

create index idx_reviews_customer
    on reviews (customer_id);

create table discounts
(
    discount_id    serial
        primary key,
    code           varchar(50),
    description    text,
    discount_value numeric(5, 2) not null
        constraint discounts_discount_value_check
            check ((discount_value > (0)::numeric) AND (discount_value <= (100)::numeric))
        constraint discount_value_check
            check ((discount_value > (0)::numeric) AND (discount_value <= (100)::numeric)),
    discount_type  varchar(20)   not null
        constraint discounts_discount_type_check
            check ((discount_type)::text = ANY
                   ((ARRAY ['PERCENTAGE'::character varying, 'FLAT'::character varying])::text[])),
    valid_from     timestamp with time zone,
    valid_until    timestamp with time zone,
    max_uses       integer default 1,
    times_used     integer default 0,
    product_id     integer
        constraint fk_product
            references products
            on delete set null,
    category_id    integer
        constraint fk_category
            references categories
            on delete set null,
    min_quantity   integer
);

create table orders
(
    order_id            serial
        primary key,
    customer_id         integer        not null
        constraint fk_customer
            references customers
            on delete restrict,
    order_date          timestamp with time zone default CURRENT_TIMESTAMP,
    total_amount        numeric(10, 2) not null,
    status              varchar(20)    not null,
    shipping_address_id integer        not null
        constraint fk_shipping_address
            references addresses
            on delete restrict,
    payment_method_id   integer        not null
        constraint fk_payment_method
            references payment_methods
            on delete restrict,
    discount_id         integer
        constraint fk_discount
            references discounts
            on delete set null
);

create index idx_orders_customer_date
    on orders (customer_id, order_date);

create table order_items
(
    order_item_id   serial
        primary key,
    order_id        integer                  not null
        constraint fk_order
            references orders
            on delete cascade,
    product_id      integer                  not null
        constraint fk_product
            references products
            on delete restrict,
    quantity        integer                  not null,
    unit_price      numeric(10, 2)           not null,
    discount_amount numeric(10, 2) default 0 not null
);

create index idx_order_items_order
    on order_items (order_id);

create index idx_order_items_product
    on order_items (product_id);

create table bills
(
    bill_id        serial
        primary key,
    order_id       integer        not null
        unique
        constraint fk_order
            references orders
            on delete restrict,
    bill_date      timestamp with time zone default CURRENT_TIMESTAMP,
    total_amount   numeric(10, 2) not null,
    payment_status varchar(20)    not null
);

create index idx_discounts_code
    on discounts (code);

create index idx_discounts_product
    on discounts (product_id);

create index idx_discounts_category
    on discounts (category_id);

create index idx_discounts_validity
    on discounts (valid_from, valid_until);

