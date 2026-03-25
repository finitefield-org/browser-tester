package runtime

func (s *Session) AddressCount() int {
	return s.elementCountForSelector("address")
}
