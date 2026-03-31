ALTER TABLE profiles ADD COLUMN role VARCHAR(20) NOT NULL DEFAULT 'user';
CREATE INDEX idx_profiles_role ON profiles(role);
