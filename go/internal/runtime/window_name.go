package runtime

import "fmt"

func (s *Session) WindowName() string {
	if s == nil {
		return ""
	}
	return s.windowName
}

func (s *Session) setWindowName(value string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.windowName = value
	return nil
}
