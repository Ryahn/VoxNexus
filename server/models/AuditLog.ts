import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IAuditLog extends Document {
  serverId: Types.ObjectId;
  action: string;
  actor: Types.ObjectId;
  target?: Types.ObjectId;
  targetType: string;
  details?: Record<string, any>;
  createdAt: Date;
}

const AuditLogSchema: Schema<IAuditLog> = new Schema({
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  action: { type: String, required: true },
  actor: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  target: { type: Schema.Types.ObjectId },
  targetType: { type: String, required: true },
  details: { type: Object },
  createdAt: { type: Date, default: Date.now },
});

export default (mongoose.models.AuditLog as Model<IAuditLog>) || mongoose.model<IAuditLog>('AuditLog', AuditLogSchema); 