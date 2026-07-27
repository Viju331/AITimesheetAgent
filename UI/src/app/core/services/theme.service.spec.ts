import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { ThemeService } from './theme.service';

describe('ThemeService', () => {
  let service: ThemeService;

  // Mock matchMedia — not present in jsdom
  const matchMediaMock = vi.fn().mockReturnValue({
    matches: false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  });

  beforeEach(() => {
    Object.defineProperty(window, 'matchMedia', { writable: true, value: matchMediaMock });
    localStorage.clear();

    TestBed.configureTestingModule({});
    service = TestBed.inject(ThemeService);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('defaults to system mode when no saved preference', () => {
    expect(service.mode()).toBe('system');
  });

  it('setMode changes the mode signal', () => {
    service.setMode('dark');
    expect(service.mode()).toBe('dark');
  });

  it('isDark is true when mode is dark', () => {
    service.setMode('dark');
    TestBed.flushEffects();
    expect(service.isDark()).toBe(true);
  });

  it('isDark is false when mode is light', () => {
    service.setMode('light');
    TestBed.flushEffects();
    expect(service.isDark()).toBe(false);
  });

  it('applies dark-theme class to body when dark', () => {
    service.setMode('dark');
    TestBed.flushEffects();
    expect(document.body.classList.contains('dark-theme')).toBe(true);
  });

  it('removes dark-theme class from body when light', () => {
    document.body.classList.add('dark-theme');
    service.setMode('light');
    TestBed.flushEffects();
    expect(document.body.classList.contains('dark-theme')).toBe(false);
  });
});
