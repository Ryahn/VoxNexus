import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IRole extends Document {
  serverId: Types.ObjectId;
  name: string;
  priority: number;
  permissions: string[];
  color?: string;
  isDefault: boolean;
}

const RoleSchema: Schema<IRole> = new Schema({
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  name: { type: String, required: true },
  priority: { type: Number, required: true },
  permissions: { type: [String], required: true },
  color: { type: String },
  isDefault: { type: Boolean, default: false },
});

export default (mongoose.models.Role as Model<IRole>) || mongoose.model<IRole>('Role', RoleSchema); 