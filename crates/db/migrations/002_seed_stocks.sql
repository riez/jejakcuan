-- Seed data for Indonesian Syariah stocks
-- Source: IDX ISSI Index (Indonesia Sharia Stock Index)
-- Updated: January 2025

-- Clear existing stock data to avoid duplicates (optional - comment out if you want to preserve existing data)
-- DELETE FROM stocks;

-- Banking Sector (Syariah compliant)
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('BBCA', 'Bank Central Asia Tbk', 'Banking', 'Bank', true),
('BMRI', 'Bank Mandiri (Persero) Tbk', 'Banking', 'Bank', true),
('BBRI', 'Bank Rakyat Indonesia (Persero) Tbk', 'Banking', 'Bank', true),
('BBNI', 'Bank Negara Indonesia (Persero) Tbk', 'Banking', 'Bank', true),
('BRIS', 'Bank Syariah Indonesia Tbk', 'Banking', 'Bank', true),
('BTPN', 'Bank BTPN Tbk', 'Banking', 'Bank', true),
('BGTG', 'Bank Ganesha Tbk', 'Banking', 'Bank', true),
('BNBA', 'Bank Bumi Arta Tbk', 'Banking', 'Bank', true),
('ARTO', 'Bank Jago Tbk', 'Banking', 'Bank', true),
('BBTN', 'Bank Tabungan Negara (Persero) Tbk', 'Banking', 'Bank', true),
('NISP', 'Bank OCBC NISP Tbk', 'Banking', 'Bank', true),
('MEGA', 'Bank Mega Tbk', 'Banking', 'Bank', true),
('PNBN', 'Bank Pan Indonesia Tbk', 'Banking', 'Bank', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Automotive & Components
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('ASII', 'Astra International Tbk', 'Industrial', 'Automotive', true),
('AUTO', 'Astra Otoparts Tbk', 'Industrial', 'Automotive', true),
('GJTL', 'Gajah Tunggal Tbk', 'Industrial', 'Automotive', true),
('SMSM', 'Selamat Sempurna Tbk', 'Industrial', 'Automotive', true),
('IMAS', 'Indomobil Sukses Internasional Tbk', 'Industrial', 'Automotive', true),
('BOLT', 'Garuda Metalindo Tbk', 'Industrial', 'Automotive', true),
('DRMA', 'Dharma Polimetal Tbk', 'Industrial', 'Automotive', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Telecommunication
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('TLKM', 'Telkom Indonesia (Persero) Tbk', 'Telecommunications', 'Telecom Services', true),
('EXCL', 'XL Axiata Tbk', 'Telecommunications', 'Telecom Services', true),
('ISAT', 'Indosat Tbk', 'Telecommunications', 'Telecom Services', true),
('FREN', 'Smartfren Telecom Tbk', 'Telecommunications', 'Telecom Services', true),
('TOWR', 'Sarana Menara Nusantara Tbk', 'Telecommunications', 'Telecom Infrastructure', true),
('TBIG', 'Tower Bersama Infrastructure Tbk', 'Telecommunications', 'Telecom Infrastructure', true),
('LINK', 'Link Net Tbk', 'Telecommunications', 'Internet Services', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Mining & Energy
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('ADRO', 'Adaro Energy Tbk', 'Mining', 'Coal', true),
('ITMG', 'Indo Tambangraya Megah Tbk', 'Mining', 'Coal', true),
('PTBA', 'Bukit Asam Tbk', 'Mining', 'Coal', true),
('INDY', 'Indika Energy Tbk', 'Mining', 'Coal', true),
('BYAN', 'Bayan Resources Tbk', 'Mining', 'Coal', true),
('HRUM', 'Harum Energy Tbk', 'Mining', 'Coal', true),
('BSSR', 'Baramulti Suksessarana Tbk', 'Mining', 'Coal', true),
('ANTM', 'Aneka Tambang Tbk', 'Mining', 'Metal', true),
('INCO', 'Vale Indonesia Tbk', 'Mining', 'Metal', true),
('TINS', 'Timah Tbk', 'Mining', 'Metal', true),
('MDKA', 'Merdeka Copper Gold Tbk', 'Mining', 'Metal', true),
('BRMS', 'Bumi Resources Minerals Tbk', 'Mining', 'Metal', true),
('MEDC', 'Medco Energi Internasional Tbk', 'Energy', 'Oil & Gas', true),
('ELSA', 'Elnusa Tbk', 'Energy', 'Oil & Gas Services', true),
('RUIS', 'Radiant Utama Interinsco Tbk', 'Energy', 'Oil & Gas Services', true),
('AKRA', 'AKR Corporindo Tbk', 'Energy', 'Energy Distribution', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Consumer Goods
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('UNVR', 'Unilever Indonesia Tbk', 'Consumer Goods', 'FMCG', true),
('ICBP', 'Indofood CBP Sukses Makmur Tbk', 'Consumer Goods', 'Food & Beverage', true),
('INDF', 'Indofood Sukses Makmur Tbk', 'Consumer Goods', 'Food & Beverage', true),
('MYOR', 'Mayora Indah Tbk', 'Consumer Goods', 'Food & Beverage', true),
('CPIN', 'Charoen Pokphand Indonesia Tbk', 'Consumer Goods', 'Poultry', true),
('HMSP', 'HM Sampoerna Tbk', 'Consumer Goods', 'Tobacco', true),
('GGRM', 'Gudang Garam Tbk', 'Consumer Goods', 'Tobacco', true),
('KLBF', 'Kalbe Farma Tbk', 'Healthcare', 'Pharmaceutical', true),
('SIDO', 'Industri Jamu dan Farmasi Sido Muncul Tbk', 'Healthcare', 'Pharmaceutical', true),
('PYFA', 'Pyridam Farma Tbk', 'Healthcare', 'Pharmaceutical', true),
('KAEF', 'Kimia Farma Tbk', 'Healthcare', 'Pharmaceutical', true),
('DVLA', 'Darya-Varia Laboratoria Tbk', 'Healthcare', 'Pharmaceutical', true),
('ADES', 'Akasha Wira International Tbk', 'Consumer Goods', 'Food & Beverage', true),
('CEKA', 'Wilmar Cahaya Indonesia Tbk', 'Consumer Goods', 'Food & Beverage', true),
('AALI', 'Astra Agro Lestari Tbk', 'Consumer Goods', 'Plantation', true),
('LSIP', 'PP London Sumatra Indonesia Tbk', 'Consumer Goods', 'Plantation', true),
('SIMP', 'Salim Ivomas Pratama Tbk', 'Consumer Goods', 'Plantation', true),
('SGRO', 'Sampoerna Agro Tbk', 'Consumer Goods', 'Plantation', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Property & Real Estate
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('BSDE', 'Bumi Serpong Damai Tbk', 'Property', 'Real Estate', true),
('CTRA', 'Ciputra Development Tbk', 'Property', 'Real Estate', true),
('SMRA', 'Summarecon Agung Tbk', 'Property', 'Real Estate', true),
('PWON', 'Pakuwon Jati Tbk', 'Property', 'Real Estate', true),
('LPKR', 'Lippo Karawaci Tbk', 'Property', 'Real Estate', true),
('DILD', 'Intiland Development Tbk', 'Property', 'Real Estate', true),
('APLN', 'Agung Podomoro Land Tbk', 'Property', 'Real Estate', true),
('PLIN', 'Plaza Indonesia Realty Tbk', 'Property', 'Real Estate', true),
('JRPT', 'Jaya Real Property Tbk', 'Property', 'Real Estate', true),
('ASRI', 'Alam Sutera Realty Tbk', 'Property', 'Real Estate', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Infrastructure & Construction
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('JSMR', 'Jasa Marga (Persero) Tbk', 'Infrastructure', 'Toll Road', true),
('WIKA', 'Wijaya Karya (Persero) Tbk', 'Infrastructure', 'Construction', true),
('WSKT', 'Waskita Karya (Persero) Tbk', 'Infrastructure', 'Construction', true),
('PTPP', 'PP (Persero) Tbk', 'Infrastructure', 'Construction', true),
('ADHI', 'Adhi Karya (Persero) Tbk', 'Infrastructure', 'Construction', true),
('WTON', 'Wijaya Karya Beton Tbk', 'Infrastructure', 'Construction Materials', true),
('ACST', 'Acset Indonusa Tbk', 'Infrastructure', 'Construction', true),
('TOTL', 'Total Bangun Persada Tbk', 'Infrastructure', 'Construction', true),
('NRCA', 'Nusa Raya Cipta Tbk', 'Infrastructure', 'Construction', true),
('SMGR', 'Semen Indonesia (Persero) Tbk', 'Basic Materials', 'Cement', true),
('INTP', 'Indocement Tunggal Prakarsa Tbk', 'Basic Materials', 'Cement', true),
('SMBR', 'Semen Baturaja (Persero) Tbk', 'Basic Materials', 'Cement', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Technology & Digital
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('GOTO', 'GoTo Gojek Tokopedia Tbk', 'Technology', 'E-commerce', true),
('BUKA', 'Bukalapak.com Tbk', 'Technology', 'E-commerce', true),
('EMTK', 'Elang Mahkota Teknologi Tbk', 'Technology', 'Media', true),
('MNCN', 'Media Nusantara Citra Tbk', 'Technology', 'Media', true),
('SCMA', 'Surya Citra Media Tbk', 'Technology', 'Media', true),
('DCII', 'DCI Indonesia Tbk', 'Technology', 'Data Center', true),
('MTDL', 'Metrodata Electronics Tbk', 'Technology', 'IT Services', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Transportation & Logistics
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('GIAA', 'Garuda Indonesia (Persero) Tbk', 'Transportation', 'Airlines', true),
('BIRD', 'Blue Bird Tbk', 'Transportation', 'Land Transport', true),
('SMDR', 'Samudera Indonesia Tbk', 'Transportation', 'Shipping', true),
('TMAS', 'Pelayaran Tempuran Emas Tbk', 'Transportation', 'Shipping', true),
('ASSA', 'Adi Sarana Armada Tbk', 'Transportation', 'Land Transport', true),
('BULL', 'Buana Lintas Lautan Tbk', 'Transportation', 'Shipping', true),
('HITS', 'Humpuss Intermoda Transportasi Tbk', 'Transportation', 'Shipping', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Retail & Trade
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('MAPI', 'Mitra Adiperkasa Tbk', 'Retail', 'Department Store', true),
('LPPF', 'Matahari Department Store Tbk', 'Retail', 'Department Store', true),
('MPPA', 'Matahari Putra Prima Tbk', 'Retail', 'Hypermarket', true),
('RALS', 'Ramayana Lestari Sentosa Tbk', 'Retail', 'Department Store', true),
('ACES', 'Ace Hardware Indonesia Tbk', 'Retail', 'Home Improvement', true),
('ERAA', 'Erajaya Swasembada Tbk', 'Retail', 'Electronics', true),
('MIDI', 'Midi Utama Indonesia Tbk', 'Retail', 'Convenience Store', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Heavy Equipment & Industrial
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('UNTR', 'United Tractors Tbk', 'Industrial', 'Heavy Equipment', true),
('PGAS', 'Perusahaan Gas Negara Tbk', 'Energy', 'Gas Distribution', true),
('INKP', 'Indah Kiat Pulp & Paper Tbk', 'Basic Materials', 'Pulp & Paper', true),
('TKIM', 'Pabrik Kertas Tjiwi Kimia Tbk', 'Basic Materials', 'Pulp & Paper', true),
('FASW', 'Fajar Surya Wisesa Tbk', 'Basic Materials', 'Pulp & Paper', true),
('JPFA', 'Japfa Comfeed Indonesia Tbk', 'Consumer Goods', 'Animal Feed', true),
('MAIN', 'Malindo Feedmill Tbk', 'Consumer Goods', 'Animal Feed', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Other Notable Stocks
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('BATA', 'Sepatu Bata Tbk', 'Consumer Goods', 'Footwear', true),
('SRIL', 'Sri Rejeki Isman Tbk', 'Consumer Goods', 'Textile', true),
('MLIA', 'Mulia Industrindo Tbk', 'Basic Materials', 'Glass', true),
('AMRT', 'Sumber Alfaria Trijaya Tbk', 'Retail', 'Convenience Store', true),
('DNET', 'Indoritel Makmur Internasional Tbk', 'Retail', 'Convenience Store', true),
('KINO', 'Kino Indonesia Tbk', 'Consumer Goods', 'Personal Care', true),
('UNSP', 'Bakrie Sumatera Plantations Tbk', 'Consumer Goods', 'Plantation', true),
('PNLF', 'Panin Financial Tbk', 'Financial', 'Insurance', true),
('BNLI', 'Bank Permata Tbk', 'Banking', 'Bank', true),
('BDMN', 'Bank Danamon Indonesia Tbk', 'Banking', 'Bank', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Additional JII30 stocks (Jakarta Islamic Index Top 30)
INSERT INTO stocks (symbol, name, sector, subsector, is_active) VALUES
('PNBN', 'Bank Pan Indonesia Tbk', 'Banking', 'Bank', true),
('SCCO', 'Supreme Cable Manufacturing & Commerce Tbk', 'Industrial', 'Cable', true),
('MDKA', 'Merdeka Copper Gold Tbk', 'Mining', 'Gold & Copper', true),
('MAPI', 'Mitra Adiperkasa Tbk', 'Retail', 'Department Store', true)
ON CONFLICT (symbol) DO UPDATE SET
  name = EXCLUDED.name,
  sector = EXCLUDED.sector,
  subsector = EXCLUDED.subsector,
  is_active = EXCLUDED.is_active,
  updated_at = NOW();

-- Log the count of stocks inserted
DO $$
DECLARE
    stock_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO stock_count FROM stocks;
    RAISE NOTICE 'Total stocks in database: %', stock_count;
END $$;
