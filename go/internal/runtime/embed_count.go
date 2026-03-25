package runtime

func (s *Session) EmbedCount() int {
	return s.elementCountForSelector("embed")
}
