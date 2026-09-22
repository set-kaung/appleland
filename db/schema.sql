--
-- PostgreSQL database dump
--

\restrict fq27K2ihjzUnWDshQo9UeCZ9fEvdZOwc2cHS8Q3zKpMtM0lkJACIbKfBWKBGR2Q

-- Dumped from database version 17.11 (Homebrew)
-- Dumped by pg_dump version 17.11 (Homebrew)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: tower_sessions; Type: SCHEMA; Schema: -; Owner: appleland_backend
--

CREATE SCHEMA tower_sessions;


ALTER SCHEMA tower_sessions OWNER TO appleland_backend;

--
-- Name: admin_role; Type: TYPE; Schema: public; Owner: appleland_backend
--

CREATE TYPE public.admin_role AS ENUM (
    'sale_admin',
    'playground_admin',
    'super_admin'
);


ALTER TYPE public.admin_role OWNER TO appleland_backend;

--
-- Name: playground_token_status; Type: TYPE; Schema: public; Owner: appleland_backend
--

CREATE TYPE public.playground_token_status AS ENUM (
    'pending',
    'claimed',
    'expired'
);


ALTER TYPE public.playground_token_status OWNER TO appleland_backend;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: admin_sales; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.admin_sales (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sale_id uuid NOT NULL,
    admin_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_sales OWNER TO appleland_backend;

--
-- Name: admins; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.admins (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    username text NOT NULL,
    password_hash text NOT NULL,
    role public.admin_role NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admins OWNER TO appleland_backend;

--
-- Name: categories; Type: TABLE; Schema: public; Owner: appleland_backend
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


ALTER TABLE public.categories OWNER TO appleland_backend;

--
-- Name: delivery_contact; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.delivery_contact (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    address_line_1 text NOT NULL,
    phone_number text NOT NULL
);


ALTER TABLE public.delivery_contact OWNER TO appleland_backend;

--
-- Name: inventory; Type: TABLE; Schema: public; Owner: appleland_backend
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


ALTER TABLE public.inventory OWNER TO appleland_backend;

--
-- Name: options; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.options (
    id bigint NOT NULL,
    question_id bigint NOT NULL,
    text text NOT NULL,
    is_correct boolean DEFAULT false NOT NULL
);


ALTER TABLE public.options OWNER TO appleland_backend;

--
-- Name: options_id_seq; Type: SEQUENCE; Schema: public; Owner: appleland_backend
--

CREATE SEQUENCE public.options_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.options_id_seq OWNER TO appleland_backend;

--
-- Name: options_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: appleland_backend
--

ALTER SEQUENCE public.options_id_seq OWNED BY public.options.id;


--
-- Name: orders; Type: TABLE; Schema: public; Owner: appleland_backend
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


ALTER TABLE public.orders OWNER TO appleland_backend;

--
-- Name: payment_webhook_nonces; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.payment_webhook_nonces (
    nonce text NOT NULL,
    consumed_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.payment_webhook_nonces OWNER TO appleland_backend;

--
-- Name: playground_accounts; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.playground_accounts (
    user_id uuid NOT NULL,
    tickets_purchased integer DEFAULT 0 NOT NULL,
    free_tickets_redeemed integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    free_ticket_eligible integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.playground_accounts OWNER TO appleland_backend;

--
-- Name: playground_coupons; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.playground_coupons (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    is_used boolean DEFAULT false NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.playground_coupons OWNER TO appleland_backend;

--
-- Name: playground_purchase_tokens; Type: TABLE; Schema: public; Owner: appleland_backend
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
    CONSTRAINT playground_purchase_tokens_quantity_check CHECK ((total_quantity > 0))
);


ALTER TABLE public.playground_purchase_tokens OWNER TO appleland_backend;

--
-- Name: playground_purchases; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.playground_purchases (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    quantity integer NOT NULL,
    is_free_redemption boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    unit_price numeric(10,2) NOT NULL
);


ALTER TABLE public.playground_purchases OWNER TO appleland_backend;

--
-- Name: products; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.products (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    barcode text NOT NULL,
    description text,
    category_id uuid,
    sub_category_id uuid,
    selling_price numeric(19,2) NOT NULL,
    is_active boolean NOT NULL,
    image_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    deleted_by uuid,
    is_perishable boolean DEFAULT false NOT NULL
);


ALTER TABLE public.products OWNER TO appleland_backend;

--
-- Name: question_answers; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.question_answers (
    id bigint NOT NULL,
    quiz_attempt_id bigint NOT NULL,
    question_id bigint NOT NULL,
    selected_option bigint NOT NULL,
    is_correct boolean NOT NULL
);


ALTER TABLE public.question_answers OWNER TO appleland_backend;

--
-- Name: question_answers_id_seq; Type: SEQUENCE; Schema: public; Owner: appleland_backend
--

CREATE SEQUENCE public.question_answers_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.question_answers_id_seq OWNER TO appleland_backend;

--
-- Name: question_answers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: appleland_backend
--

ALTER SEQUENCE public.question_answers_id_seq OWNED BY public.question_answers.id;


--
-- Name: questions; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.questions (
    id bigint NOT NULL,
    quiz_id bigint NOT NULL,
    text text NOT NULL
);


ALTER TABLE public.questions OWNER TO appleland_backend;

--
-- Name: questions_id_seq; Type: SEQUENCE; Schema: public; Owner: appleland_backend
--

CREATE SEQUENCE public.questions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.questions_id_seq OWNER TO appleland_backend;

--
-- Name: questions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: appleland_backend
--

ALTER SEQUENCE public.questions_id_seq OWNED BY public.questions.id;


--
-- Name: quiz_attempts; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.quiz_attempts (
    id bigint NOT NULL,
    quiz_id bigint NOT NULL,
    score bigint NOT NULL,
    total_questions bigint NOT NULL,
    created_at timestamp with time zone
);


ALTER TABLE public.quiz_attempts OWNER TO appleland_backend;

--
-- Name: quiz_attempts_id_seq; Type: SEQUENCE; Schema: public; Owner: appleland_backend
--

CREATE SEQUENCE public.quiz_attempts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.quiz_attempts_id_seq OWNER TO appleland_backend;

--
-- Name: quiz_attempts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: appleland_backend
--

ALTER SEQUENCE public.quiz_attempts_id_seq OWNED BY public.quiz_attempts.id;


--
-- Name: quiz_tags; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.quiz_tags (
    quiz_id bigint NOT NULL,
    tag_id bigint NOT NULL
);


ALTER TABLE public.quiz_tags OWNER TO appleland_backend;

--
-- Name: quizzes; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.quizzes (
    id bigint NOT NULL,
    title text NOT NULL,
    created_at timestamp with time zone,
    updated_at timestamp with time zone
);


ALTER TABLE public.quizzes OWNER TO appleland_backend;

--
-- Name: quizzes_id_seq; Type: SEQUENCE; Schema: public; Owner: appleland_backend
--

CREATE SEQUENCE public.quizzes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.quizzes_id_seq OWNER TO appleland_backend;

--
-- Name: quizzes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: appleland_backend
--

ALTER SEQUENCE public.quizzes_id_seq OWNED BY public.quizzes.id;


--
-- Name: sale_items; Type: TABLE; Schema: public; Owner: appleland_backend
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


ALTER TABLE public.sale_items OWNER TO appleland_backend;

--
-- Name: sales; Type: TABLE; Schema: public; Owner: appleland_backend
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


ALTER TABLE public.sales OWNER TO appleland_backend;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.tags (
    id bigint NOT NULL,
    name text NOT NULL
);


ALTER TABLE public.tags OWNER TO appleland_backend;

--
-- Name: tags_id_seq; Type: SEQUENCE; Schema: public; Owner: appleland_backend
--

CREATE SEQUENCE public.tags_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tags_id_seq OWNER TO appleland_backend;

--
-- Name: tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: appleland_backend
--

ALTER SEQUENCE public.tags_id_seq OWNED BY public.tags.id;


--
-- Name: user_contact; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.user_contact (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    address_line_1 text NOT NULL,
    phone_number text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    user_id uuid NOT NULL
);


ALTER TABLE public.user_contact OWNER TO appleland_backend;

--
-- Name: user_sales; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.user_sales (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    sale_id uuid NOT NULL
);


ALTER TABLE public.user_sales OWNER TO appleland_backend;

--
-- Name: users; Type: TABLE; Schema: public; Owner: appleland_backend
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    supabase_id uuid
);


ALTER TABLE public.users OWNER TO appleland_backend;

--
-- Name: session; Type: TABLE; Schema: tower_sessions; Owner: appleland_backend
--

CREATE TABLE tower_sessions.session (
    id text NOT NULL,
    data bytea NOT NULL,
    expiry_date timestamp with time zone NOT NULL
);


ALTER TABLE tower_sessions.session OWNER TO appleland_backend;

--
-- Name: options id; Type: DEFAULT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.options ALTER COLUMN id SET DEFAULT nextval('public.options_id_seq'::regclass);


--
-- Name: question_answers id; Type: DEFAULT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.question_answers ALTER COLUMN id SET DEFAULT nextval('public.question_answers_id_seq'::regclass);


--
-- Name: questions id; Type: DEFAULT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.questions ALTER COLUMN id SET DEFAULT nextval('public.questions_id_seq'::regclass);


--
-- Name: quiz_attempts id; Type: DEFAULT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quiz_attempts ALTER COLUMN id SET DEFAULT nextval('public.quiz_attempts_id_seq'::regclass);


--
-- Name: quizzes id; Type: DEFAULT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quizzes ALTER COLUMN id SET DEFAULT nextval('public.quizzes_id_seq'::regclass);


--
-- Name: tags id; Type: DEFAULT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq'::regclass);


--
-- Name: admin_sales admin_sales_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.admin_sales
    ADD CONSTRAINT admin_sales_pkey PRIMARY KEY (id);


--
-- Name: admins admins_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.admins
    ADD CONSTRAINT admins_pkey PRIMARY KEY (id);


--
-- Name: categories categories_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_pkey PRIMARY KEY (id);


--
-- Name: inventory inventory_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.inventory
    ADD CONSTRAINT inventory_pkey PRIMARY KEY (id);


--
-- Name: options options_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.options
    ADD CONSTRAINT options_pkey PRIMARY KEY (id);


--
-- Name: orders orders_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);


--
-- Name: orders orders_sale_id_key; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_sale_id_key UNIQUE (sale_id);


--
-- Name: payment_webhook_nonces payment_webhook_nonces_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.payment_webhook_nonces
    ADD CONSTRAINT payment_webhook_nonces_pkey PRIMARY KEY (nonce);


--
-- Name: playground_accounts playground_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_accounts
    ADD CONSTRAINT playground_accounts_pkey PRIMARY KEY (user_id);


--
-- Name: playground_coupons playground_coupons_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_coupons
    ADD CONSTRAINT playground_coupons_pkey PRIMARY KEY (id);


--
-- Name: playground_purchase_tokens playground_purchase_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_purchase_tokens
    ADD CONSTRAINT playground_purchase_tokens_pkey PRIMARY KEY (id);


--
-- Name: playground_purchases playground_purchases_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_pkey PRIMARY KEY (id);


--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);


--
-- Name: question_answers question_answers_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.question_answers
    ADD CONSTRAINT question_answers_pkey PRIMARY KEY (id);


--
-- Name: questions questions_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.questions
    ADD CONSTRAINT questions_pkey PRIMARY KEY (id);


--
-- Name: quiz_attempts quiz_attempts_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quiz_attempts
    ADD CONSTRAINT quiz_attempts_pkey PRIMARY KEY (id);


--
-- Name: quiz_tags quiz_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quiz_tags
    ADD CONSTRAINT quiz_tags_pkey PRIMARY KEY (quiz_id, tag_id);


--
-- Name: quizzes quizzes_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quizzes
    ADD CONSTRAINT quizzes_pkey PRIMARY KEY (id);


--
-- Name: sale_items sale_items_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT sale_items_pkey PRIMARY KEY (id);


--
-- Name: sales sales_payment_order_id_key; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.sales
    ADD CONSTRAINT sales_payment_order_id_key UNIQUE (payment_order_id);


--
-- Name: sales sales_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.sales
    ADD CONSTRAINT sales_pkey PRIMARY KEY (id);


--
-- Name: delivery_contact shipping_addresses_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.delivery_contact
    ADD CONSTRAINT shipping_addresses_pkey PRIMARY KEY (id);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: user_contact user_contact_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.user_contact
    ADD CONSTRAINT user_contact_pkey PRIMARY KEY (id);


--
-- Name: user_sales user_sale_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.user_sales
    ADD CONSTRAINT user_sale_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_supabase_id_key; Type: CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_supabase_id_key UNIQUE (supabase_id);


--
-- Name: session session_pkey; Type: CONSTRAINT; Schema: tower_sessions; Owner: appleland_backend
--

ALTER TABLE ONLY tower_sessions.session
    ADD CONSTRAINT session_pkey PRIMARY KEY (id);


--
-- Name: idx_attempt_question; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE UNIQUE INDEX idx_attempt_question ON public.question_answers USING btree (quiz_attempt_id, question_id);


--
-- Name: idx_delivery_contact_user_id; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_delivery_contact_user_id ON public.user_contact USING btree (user_id);


--
-- Name: idx_options_question_id; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_options_question_id ON public.options USING btree (question_id);


--
-- Name: idx_payment_webhook_nonces_consumed_at; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_payment_webhook_nonces_consumed_at ON public.payment_webhook_nonces USING btree (consumed_at);


--
-- Name: idx_products_barcode_active; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE UNIQUE INDEX idx_products_barcode_active ON public.products USING btree (barcode) WHERE (is_deleted IS FALSE);


--
-- Name: idx_questions_quiz_id; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_questions_quiz_id ON public.questions USING btree (quiz_id);


--
-- Name: idx_quiz_created; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_quiz_created ON public.quiz_attempts USING btree (quiz_id, created_at);


--
-- Name: idx_sale_items_product_id; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_sale_items_product_id ON public.sale_items USING btree (product_id);


--
-- Name: idx_sales_admin; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_sales_admin ON public.admin_sales USING btree (admin_id);


--
-- Name: idx_sales_created_at; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_sales_created_at ON public.sales USING btree (created_at);


--
-- Name: idx_tags_name; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE UNIQUE INDEX idx_tags_name ON public.tags USING btree (name);


--
-- Name: idx_tokens_status_expires; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_tokens_status_expires ON public.playground_purchase_tokens USING btree (status, expires_at);


--
-- Name: idx_user_sales_user_id; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_user_sales_user_id ON public.user_sales USING btree (user_id);


--
-- Name: idx_users_supabase_id; Type: INDEX; Schema: public; Owner: appleland_backend
--

CREATE INDEX idx_users_supabase_id ON public.users USING btree (id);


--
-- Name: admin_sales admin_sale_admin_fk; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.admin_sales
    ADD CONSTRAINT admin_sale_admin_fk FOREIGN KEY (admin_id) REFERENCES public.admins(id);


--
-- Name: admin_sales admin_sale_sale_fk; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.admin_sales
    ADD CONSTRAINT admin_sale_sale_fk FOREIGN KEY (sale_id) REFERENCES public.sales(id);


--
-- Name: categories fk_deleted_by; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT fk_deleted_by FOREIGN KEY (deleted_by) REFERENCES public.admins(id);


--
-- Name: playground_coupons fk_playground_coupon_user; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_coupons
    ADD CONSTRAINT fk_playground_coupon_user FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: inventory fk_products_created_by; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.inventory
    ADD CONSTRAINT fk_products_created_by FOREIGN KEY (created_by) REFERENCES public.admins(id);


--
-- Name: products fk_products_created_by; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT fk_products_created_by FOREIGN KEY (created_by) REFERENCES public.admins(id);


--
-- Name: inventory fk_products_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.inventory
    ADD CONSTRAINT fk_products_updated_by FOREIGN KEY (updated_by) REFERENCES public.admins(id);


--
-- Name: products fk_products_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT fk_products_updated_by FOREIGN KEY (updated_by) REFERENCES public.admins(id);


--
-- Name: options fk_questions_options; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.options
    ADD CONSTRAINT fk_questions_options FOREIGN KEY (question_id) REFERENCES public.questions(id);


--
-- Name: quiz_tags fk_quiz_tags_quiz; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quiz_tags
    ADD CONSTRAINT fk_quiz_tags_quiz FOREIGN KEY (quiz_id) REFERENCES public.quizzes(id);


--
-- Name: quiz_tags fk_quiz_tags_tag; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.quiz_tags
    ADD CONSTRAINT fk_quiz_tags_tag FOREIGN KEY (tag_id) REFERENCES public.tags(id);


--
-- Name: questions fk_quizzes_questions; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.questions
    ADD CONSTRAINT fk_quizzes_questions FOREIGN KEY (quiz_id) REFERENCES public.quizzes(id);


--
-- Name: sale_items fk_sale_items_sale; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT fk_sale_items_sale FOREIGN KEY (sale_id) REFERENCES public.sales(id);


--
-- Name: sale_items fk_sale_product; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT fk_sale_product FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: user_sales fk_user_sales_sale_id; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.user_sales
    ADD CONSTRAINT fk_user_sales_sale_id FOREIGN KEY (sale_id) REFERENCES public.sales(id);


--
-- Name: playground_accounts playground_accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_accounts
    ADD CONSTRAINT playground_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: playground_purchase_tokens playground_purchase_tokens_claimed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_purchase_tokens
    ADD CONSTRAINT playground_purchase_tokens_claimed_by_fkey FOREIGN KEY (claimed_by) REFERENCES public.users(id);


--
-- Name: playground_purchase_tokens playground_purchase_tokens_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_purchase_tokens
    ADD CONSTRAINT playground_purchase_tokens_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.admins(id);


--
-- Name: playground_purchases playground_purchases_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.admins(id);


--
-- Name: playground_purchases playground_purchases_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.playground_purchases
    ADD CONSTRAINT playground_purchases_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: products products_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.categories(id);


--
-- Name: products products_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.admins(id);


--
-- Name: products products_sub_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_sub_category_id_fkey FOREIGN KEY (sub_category_id) REFERENCES public.categories(id);


--
-- Name: orders sale_fk; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT sale_fk FOREIGN KEY (sale_id) REFERENCES public.sales(id);


--
-- Name: sale_items sale_items_inventory_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.sale_items
    ADD CONSTRAINT sale_items_inventory_id_fkey FOREIGN KEY (inventory_id) REFERENCES public.inventory(id);


--
-- Name: orders shipping_address_fk; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT shipping_address_fk FOREIGN KEY (shipping_address_id) REFERENCES public.delivery_contact(id);


--
-- Name: user_contact user_contact_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.user_contact
    ADD CONSTRAINT user_contact_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: user_sales user_sale_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: appleland_backend
--

ALTER TABLE ONLY public.user_sales
    ADD CONSTRAINT user_sale_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: pg_database_owner
--

GRANT ALL ON SCHEMA public TO appleland_backend;


--
-- Name: DEFAULT PRIVILEGES FOR SEQUENCES; Type: DEFAULT ACL; Schema: public; Owner: setkaung
--

ALTER DEFAULT PRIVILEGES FOR ROLE setkaung IN SCHEMA public GRANT ALL ON SEQUENCES TO appleland_backend;


--
-- Name: DEFAULT PRIVILEGES FOR FUNCTIONS; Type: DEFAULT ACL; Schema: public; Owner: setkaung
--

ALTER DEFAULT PRIVILEGES FOR ROLE setkaung IN SCHEMA public GRANT ALL ON FUNCTIONS TO appleland_backend;


--
-- Name: DEFAULT PRIVILEGES FOR TABLES; Type: DEFAULT ACL; Schema: public; Owner: setkaung
--

ALTER DEFAULT PRIVILEGES FOR ROLE setkaung IN SCHEMA public GRANT ALL ON TABLES TO appleland_backend;


--
-- PostgreSQL database dump complete
--

\unrestrict fq27K2ihjzUnWDshQo9UeCZ9fEvdZOwc2cHS8Q3zKpMtM0lkJACIbKfBWKBGR2Q

