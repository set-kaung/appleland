--
-- PostgreSQL database dump
--

-- Dumped from database version 17.11 (Homebrew)
-- Dumped by pg_dump version 17.11 (Homebrew)

--
-- Name: tower_sessions; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA tower_sessions;

--
-- Name: admin_role; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.admin_role AS ENUM (
    'sale_admin',
    'playground_admin',
    'super_admin'
);

--
-- Name: playground_token_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.playground_token_status AS ENUM (
    'pending',
    'claimed',
    'expired'
);

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);

--
-- Name: admin_ai_chat_usage; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.admin_ai_chat_usage (
    admin_id uuid NOT NULL,
    usage_date date NOT NULL,
    request_count integer DEFAULT 0 NOT NULL
);

--
-- Name: admin_sales; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.admin_sales (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sale_id uuid NOT NULL,
    admin_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: admins; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.admins (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    username text NOT NULL,
    password_hash text NOT NULL,
    role public.admin_role NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.categories (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_by uuid NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    deleted_by uuid
);

--
-- Name: delivery_contact; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.delivery_contact (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    address_line_1 text NOT NULL,
    phone_number text NOT NULL
);

--
-- Name: inventory; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.inventory (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    product_id uuid NOT NULL,
    quantity_received integer NOT NULL,
    quantity_remaining integer NOT NULL,
    unit_cost numeric(19,2) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    received_at timestamp with time zone DEFAULT now() NOT NULL,
    expiry_date timestamp with time zone,
    CONSTRAINT check_remaining CHECK ((quantity_remaining >= 0))
);

--
-- Name: order_status_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.order_status_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    order_id uuid NOT NULL,
    status text NOT NULL,
    changed_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: orders; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.orders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sale_id uuid NOT NULL,
    shipping_address_id uuid NOT NULL,
    order_method text DEFAULT 'app order'::text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_by uuid NOT NULL,
    order_tracking_url text,
    delivery_status text DEFAULT 'pending'::text NOT NULL
);

--
-- Name: payment_webhook_nonces; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.payment_webhook_nonces (
    nonce text NOT NULL,
    consumed_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: playground_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.playground_accounts (
    user_id uuid NOT NULL,
    tickets_purchased integer DEFAULT 0 NOT NULL,
    free_tickets_redeemed integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    free_ticket_eligible integer DEFAULT 0 NOT NULL
);

--
-- Name: playground_coupons; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.playground_coupons (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    is_used boolean DEFAULT false NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: playground_purchase_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.playground_purchase_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    total_quantity integer NOT NULL,
    status public.playground_token_status DEFAULT 'pending'::public.playground_token_status NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    claimed_by uuid,
    claimed_at timestamp with time zone,
    unit_price numeric(10,2) NOT NULL,
    free_quantity integer DEFAULT 0 NOT NULL,
    CONSTRAINT playground_purchase_tokens_quantity_check CHECK ((total_quantity > 0))
);

--
-- Name: playground_purchases; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.playground_purchases (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    quantity integer NOT NULL,
    is_free_redemption boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    unit_price numeric(10,2) NOT NULL,
    token_id uuid NOT NULL
);

--
-- Name: products; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.products (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    barcode text NOT NULL,
    description text,
    category_id uuid,
    sub_category_id uuid,
    selling_price numeric(19,2) NOT NULL,
    is_shown_online boolean NOT NULL,
    image_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    deleted_by uuid,
    is_perishable boolean DEFAULT false NOT NULL,
    original_price numeric(19,2),
    CONSTRAINT products_original_price_check CHECK (((original_price IS NULL) OR (original_price > selling_price)))
);

--
-- Name: sale_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sale_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sale_id uuid NOT NULL,
    product_id uuid NOT NULL,
    quantity integer NOT NULL,
    inventory_id uuid NOT NULL,
    selling_price numeric(19,2) NOT NULL,
    cost_price numeric(19,2) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT sale_items_cost_price_check CHECK ((cost_price >= (0)::numeric)),
    CONSTRAINT sale_items_quantity_check CHECK ((quantity > 0)),
    CONSTRAINT sale_items_selling_price_check CHECK ((selling_price >= (0)::numeric))
);

--
-- Name: sales; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sales (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    total_amount numeric(20,2) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_instore boolean NOT NULL,
    payment_order_id text,
    payment_method text,
    payment_provider text
);

--
-- Name: settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.settings (
    key text NOT NULL,
    value jsonb NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    type text DEFAULT 'string'::text NOT NULL,
    CONSTRAINT settings_value_type_check CHECK ((type = ANY (ARRAY['string'::text, 'integer'::text, 'decimal'::text, 'boolean'::text])))
);

--
-- Name: user_contact; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_contact (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    address_line_1 text NOT NULL,
    phone_number text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    user_id uuid NOT NULL
);

--
-- Name: user_sales; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_sales (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    sale_id uuid NOT NULL
);

--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    supabase_id uuid,
    username text NOT NULL
);

--
-- Name: session; Type: TABLE; Schema: tower_sessions; Owner: -
--

CREATE TABLE tower_sessions.session (
    id text NOT NULL,
    data bytea NOT NULL,
    expiry_date timestamp with time zone NOT NULL
);

--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);

--
-- Name: admin_ai_chat_usage admin_ai_chat_usage_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_ai_chat_usage
    ADD CONSTRAINT admin_ai_chat_usage_pkey PRIMARY KEY (admin_id, usage_date);

--
-- Name: admin_sales admin_sales_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_sales
    ADD CONSTRAINT admin_sales_pkey PRIMARY KEY (id);

--
-- Name: admins admins_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admins
    ADD CONSTRAINT admins_pkey PRIMARY KEY (id);

--
-- Name: categories categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_pkey PRIMARY KEY (id);

--
-- Name: inventory inventory_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.inventory
    ADD CONSTRAINT inventory_pkey PRIMARY KEY (id);

--
-- Name: order_status_history order_status_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.order_status_history
    ADD CONSTRAINT order_status_history_pkey PRIMARY KEY (id);

--
-- Name: orders orders_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);

--
-- Name: orders orders_sale_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_sale_id_key UNIQUE (sale_id);

--
-- Name: payment_webhook_nonces payment_webhook_nonces_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_webhook_nonces
    ADD CONSTRAINT payment_webhook_nonces_pkey PRIMARY KEY (nonce);

--
-- Name: playground_accounts playground_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_accounts
    ADD CONSTRAINT playground_accounts_pkey PRIMARY KEY (user_id);

--
-- Name: playground_coupons playground_coupons_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_coupons
    ADD CONSTRAINT playground_coupons_pkey PRIMARY KEY (id);

--
-- Name: playground_purchase_tokens playground_purchase_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchase_tokens
    ADD CONSTRAINT playground_purchase_tokens_pkey PRIMARY KEY (id);

--
-- Name: playground_purchases playground_purchases_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_pkey PRIMARY KEY (id);

--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);

--
-- Name: sale_items sale_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT sale_items_pkey PRIMARY KEY (id);

--
-- Name: sales sales_payment_order_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sales
    ADD CONSTRAINT sales_payment_order_id_key UNIQUE (payment_order_id);

--
-- Name: sales sales_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sales
    ADD CONSTRAINT sales_pkey PRIMARY KEY (id);

--
-- Name: settings settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.settings
    ADD CONSTRAINT settings_pkey PRIMARY KEY (key);

--
-- Name: delivery_contact shipping_addresses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.delivery_contact
    ADD CONSTRAINT shipping_addresses_pkey PRIMARY KEY (id);

--
-- Name: user_contact user_contact_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_contact
    ADD CONSTRAINT user_contact_pkey PRIMARY KEY (id);

--
-- Name: user_sales user_sale_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sales
    ADD CONSTRAINT user_sale_pkey PRIMARY KEY (id);

--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);

--
-- Name: users users_supabase_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_supabase_id_key UNIQUE (supabase_id);

--
-- Name: session session_pkey; Type: CONSTRAINT; Schema: tower_sessions; Owner: -
--

ALTER TABLE ONLY tower_sessions.session
    ADD CONSTRAINT session_pkey PRIMARY KEY (id);

--
-- Name: admin_sales_sale_id_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX admin_sales_sale_id_unique ON public.admin_sales USING btree (sale_id);

--
-- Name: idx_delivery_contact_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_delivery_contact_user_id ON public.user_contact USING btree (user_id);

--
-- Name: idx_order_status_history_order_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_order_status_history_order_created ON public.order_status_history USING btree (order_id, created_at);

--
-- Name: idx_payment_webhook_nonces_consumed_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payment_webhook_nonces_consumed_at ON public.payment_webhook_nonces USING btree (consumed_at);

--
-- Name: idx_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_id ON public.inventory USING btree (product_id);

--
-- Name: idx_products_barcode_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_products_barcode_active ON public.products USING btree (barcode) WHERE (is_deleted IS FALSE);

--
-- Name: idx_sale_items_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sale_items_product_id ON public.sale_items USING btree (product_id);

--
-- Name: idx_sales_admin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sales_admin ON public.admin_sales USING btree (admin_id);

--
-- Name: idx_sales_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sales_created_at ON public.sales USING btree (created_at);

--
-- Name: idx_tokens_status_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tokens_status_expires ON public.playground_purchase_tokens USING btree (status, expires_at);

--
-- Name: idx_user_sales_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sales_user_id ON public.user_sales USING btree (user_id);

--
-- Name: idx_users_supabase_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_supabase_id ON public.users USING btree (id);

--
-- Name: user_sales_sale_id_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX user_sales_sale_id_unique ON public.user_sales USING btree (sale_id);

--
-- Name: admin_ai_chat_usage admin_ai_chat_usage_admin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_ai_chat_usage
    ADD CONSTRAINT admin_ai_chat_usage_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.admins(id);

--
-- Name: admin_sales admin_sale_admin_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_sales
    ADD CONSTRAINT admin_sale_admin_fk FOREIGN KEY (admin_id) REFERENCES public.admins(id);

--
-- Name: admin_sales admin_sale_sale_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_sales
    ADD CONSTRAINT admin_sale_sale_fk FOREIGN KEY (sale_id) REFERENCES public.sales(id);

--
-- Name: categories fk_deleted_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT fk_deleted_by FOREIGN KEY (deleted_by) REFERENCES public.admins(id);

--
-- Name: playground_coupons fk_playground_coupon_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_coupons
    ADD CONSTRAINT fk_playground_coupon_user FOREIGN KEY (user_id) REFERENCES public.users(id);

--
-- Name: inventory fk_products_created_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.inventory
    ADD CONSTRAINT fk_products_created_by FOREIGN KEY (created_by) REFERENCES public.admins(id);

--
-- Name: products fk_products_created_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT fk_products_created_by FOREIGN KEY (created_by) REFERENCES public.admins(id);

--
-- Name: inventory fk_products_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.inventory
    ADD CONSTRAINT fk_products_updated_by FOREIGN KEY (updated_by) REFERENCES public.admins(id);

--
-- Name: products fk_products_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT fk_products_updated_by FOREIGN KEY (updated_by) REFERENCES public.admins(id);

--
-- Name: sale_items fk_sale_items_sale; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT fk_sale_items_sale FOREIGN KEY (sale_id) REFERENCES public.sales(id);

--
-- Name: sale_items fk_sale_product; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT fk_sale_product FOREIGN KEY (product_id) REFERENCES public.products(id);

--
-- Name: user_sales fk_user_sales_sale_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sales
    ADD CONSTRAINT fk_user_sales_sale_id FOREIGN KEY (sale_id) REFERENCES public.sales(id);

--
-- Name: order_status_history order_status_history_order_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.order_status_history
    ADD CONSTRAINT order_status_history_order_id_fkey FOREIGN KEY (order_id) REFERENCES public.orders(id) ON DELETE CASCADE;

--
-- Name: playground_accounts playground_accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_accounts
    ADD CONSTRAINT playground_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);

--
-- Name: playground_purchase_tokens playground_purchase_tokens_claimed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchase_tokens
    ADD CONSTRAINT playground_purchase_tokens_claimed_by_fkey FOREIGN KEY (claimed_by) REFERENCES public.users(id);

--
-- Name: playground_purchase_tokens playground_purchase_tokens_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchase_tokens
    ADD CONSTRAINT playground_purchase_tokens_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.admins(id);

--
-- Name: playground_purchases playground_purchases_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.admins(id);

--
-- Name: playground_purchases playground_purchases_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_token_id_fkey FOREIGN KEY (token_id) REFERENCES public.playground_purchase_tokens(id);

--
-- Name: playground_purchases playground_purchases_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);

--
-- Name: products products_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.categories(id);

--
-- Name: products products_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.admins(id);

--
-- Name: products products_sub_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_sub_category_id_fkey FOREIGN KEY (sub_category_id) REFERENCES public.categories(id);

--
-- Name: orders sale_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT sale_fk FOREIGN KEY (sale_id) REFERENCES public.sales(id);

--
-- Name: sale_items sale_items_inventory_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT sale_items_inventory_id_fkey FOREIGN KEY (inventory_id) REFERENCES public.inventory(id);

--
-- Name: settings settings_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.settings
    ADD CONSTRAINT settings_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.admins(id);

--
-- Name: settings settings_updaetd_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.settings
    ADD CONSTRAINT settings_updaetd_by_fkey FOREIGN KEY (updated_by) REFERENCES public.admins(id);

--
-- Name: orders shipping_address_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT shipping_address_fk FOREIGN KEY (shipping_address_id) REFERENCES public.delivery_contact(id);

--
-- Name: user_contact user_contact_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_contact
    ADD CONSTRAINT user_contact_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);

--
-- Name: user_sales user_sale_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sales
    ADD CONSTRAINT user_sale_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);

--
-- PostgreSQL database dump complete
--

