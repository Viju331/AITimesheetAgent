import { describe, it, expect, beforeEach } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { UserStore } from './user.store';

describe('UserStore', () => {
  let store: UserStore;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    store = TestBed.inject(UserStore);
  });

  it('starts with no user', () => {
    expect(store.hasUser()).toBe(false);
    expect(store.isLoaded()).toBe(false);
  });

  it('setUser populates signals', () => {
    store.setUser({ id: '1', name: 'Vijay', email: 'v@test.com' });
    expect(store.hasUser()).toBe(true);
    expect(store.name()).toBe('Vijay');
    expect(store.email()).toBe('v@test.com');
    expect(store.isLoaded()).toBe(true);
  });

  it('clear resets all signals', () => {
    store.setUser({ id: '1', name: 'Vijay', email: null });
    store.clear();
    expect(store.hasUser()).toBe(false);
    expect(store.name()).toBeNull();
  });
});
