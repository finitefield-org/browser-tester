package runtime

func (s *Session) NavCount() int {
	return s.elementCountForSelector("nav")
}
