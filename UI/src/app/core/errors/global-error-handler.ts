import { ErrorHandler, Injectable, inject } from '@angular/core';
import { ErrorService } from './error.service';

@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
  private readonly errorService = inject(ErrorService);

  handleError(error: unknown): void {
    this.errorService.handle(error, 'GlobalErrorHandler');
  }
}
