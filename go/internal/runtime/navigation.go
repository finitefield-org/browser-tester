package runtime

func (s *Session) NavigationLog() []string {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	location := s.Registry().Location()
	if location == nil {
		return nil
	}
	return location.Navigations()
}
