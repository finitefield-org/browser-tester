package runtime

import (
	"fmt"

	"browsertester/internal/mocks"
)

func (s *Session) LocalStorage() map[string]string {
	if s == nil {
		return nil
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return nil
	}
	return storage.Local()
}

func (s *Session) SessionStorage() map[string]string {
	if s == nil {
		return nil
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return nil
	}
	return storage.Session()
}

func (s *Session) StorageEvents() []mocks.StorageEvent {
	if s == nil {
		return nil
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return nil
	}
	return storage.Events()
}

func (s *Session) localStorageGetItem(key string) (string, bool) {
	return s.storageGetItem("local", key)
}

func (s *Session) sessionStorageGetItem(key string) (string, bool) {
	return s.storageGetItem("session", key)
}

func (s *Session) localStorageSetItem(key, value string) error {
	return s.storageSetItem("local", key, value)
}

func (s *Session) sessionStorageSetItem(key, value string) error {
	return s.storageSetItem("session", key, value)
}

func (s *Session) localStorageRemoveItem(key string) error {
	return s.storageRemoveItem("local", key)
}

func (s *Session) sessionStorageRemoveItem(key string) error {
	return s.storageRemoveItem("session", key)
}

func (s *Session) localStorageClear() error {
	return s.storageClear("local")
}

func (s *Session) sessionStorageClear() error {
	return s.storageClear("session")
}

func (s *Session) localStorageLength() int {
	return s.storageLength("local")
}

func (s *Session) sessionStorageLength() int {
	return s.storageLength("session")
}

func (s *Session) localStorageKey(index int) (string, bool) {
	return s.storageKey("local", index)
}

func (s *Session) sessionStorageKey(index int) (string, bool) {
	return s.storageKey("session", index)
}

func (s *Session) storageGetItem(scope, key string) (string, bool) {
	if s == nil {
		return "", false
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return "", false
	}
	return storage.Get(scope, key)
}

func (s *Session) storageSetItem(scope, key, value string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return fmt.Errorf("storage registry is unavailable")
	}
	if ok := storage.Set(scope, key, value); !ok {
		return fmt.Errorf("unsupported storage scope %q", scope)
	}
	return nil
}

func (s *Session) storageRemoveItem(scope, key string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return fmt.Errorf("storage registry is unavailable")
	}
	if ok := storage.Remove(scope, key); !ok {
		return fmt.Errorf("unsupported storage scope %q", scope)
	}
	return nil
}

func (s *Session) storageClear(scope string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return fmt.Errorf("storage registry is unavailable")
	}
	if ok := storage.Clear(scope); !ok {
		return fmt.Errorf("unsupported storage scope %q", scope)
	}
	return nil
}

func (s *Session) storageLength(scope string) int {
	if s == nil {
		return 0
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return 0
	}
	length, ok := storage.Length(scope)
	if !ok {
		return 0
	}
	return length
}

func (s *Session) storageKey(scope string, index int) (string, bool) {
	if s == nil {
		return "", false
	}
	storage := s.Registry().Storage()
	if storage == nil {
		return "", false
	}
	return storage.Key(scope, index)
}
