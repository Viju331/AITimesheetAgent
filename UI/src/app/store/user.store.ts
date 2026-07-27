import { Injectable, signal, computed } from '@angular/core';

export interface UserState {
  id:        string | null;
  name:      string | null;
  email:     string | null;
  loaded:    boolean;
}

const initialState: UserState = {
  id:     null,
  name:   null,
  email:  null,
  loaded: false,
};

@Injectable({ providedIn: 'root' })
export class UserStore {
  private readonly _state = signal<UserState>(initialState);

  readonly state    = this._state.asReadonly();
  readonly id       = computed(() => this._state().id);
  readonly name     = computed(() => this._state().name);
  readonly email    = computed(() => this._state().email);
  readonly isLoaded = computed(() => this._state().loaded);
  readonly hasUser  = computed(() => this._state().id !== null);

  setUser(user: { id: string; name: string; email: string | null }): void {
    this._state.set({ ...user, loaded: true });
  }

  clear(): void {
    this._state.set(initialState);
  }
}
